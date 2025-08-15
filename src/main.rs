/// Taking pictures
mod camera;
/// Getting endpoints to web server
mod endpoints;
/// Received message handlers
mod functions;
/// NTP Synchronization
mod ntp_sync;
/// Settings structs
mod settings;
/// Startup - before listening to messages in loop
mod startup;
/// Auto updater
mod updater;
/// Misc
mod utils;

use crate::functions::handle_notification;
use crate::startup::{critical_startup, startup};
use crate::updater::restart;
use crate::utils::{AsyncClientExt, PublishExt, SuccessWrapper};
use nix::sys::time::TimeValLike;
use rumqttc::v5::Event;
use rumqttc::v5::mqttbytes::v5::Packet;
use std::env;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::signal;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

// const VERSION: &str = concat!("MYAPP_VERSION=", env!("CARGO_PKG_VERSION"));
pub const MYAPPVERSION: &'static str = concat!("[MYAPPVERSION:", env!("CARGO_PKG_VERSION"), "]");

#[tokio::main]
async fn main() {
    println!("Version: {}", MYAPPVERSION);
    // Critical startup
    let (base_settings, mqtt_client, mut mqtt_event_loop, http_client, current_exe) =
        critical_startup().await;

    let (settings, camera_service) = startup(&base_settings, &mqtt_client).await;

    // Reference counting
    let base_settings = Arc::new(base_settings);
    let settings = Arc::new(settings);
    let mqtt_client = Arc::new(mqtt_client);
    let http_client = Arc::new(http_client);
    let should_restart = Arc::new(AtomicBool::new(false));
    let camera_service = Arc::new(Mutex::new(camera_service));
    let current_exe = Arc::new(current_exe);

    let mqtt_loop = async {
        let mut join_set = JoinSet::new();
        loop {
            // Restart, if needed
            if should_restart.load(Ordering::Relaxed) {
                let current_exe = Arc::clone(&current_exe);
                restart(&current_exe);
                break;
            }

            // Clean up tasks
            while let Some(res) = join_set.try_join_next() {
                match res {
                    Ok(_) => { /* task completed successfully, cleaned up */ }
                    Err(e) => eprintln!("Task failed: {:?}", e),
                }
            }

            // Wait for message
            let notification = mqtt_event_loop.poll().await;
            let wall_nanoseconds = nix::time::clock_gettime(nix::time::ClockId::CLOCK_REALTIME)
                .ok()
                .map(|wall_time| wall_time.num_nanoseconds());

            // Only do bare minimum, spawn task when possible
            match notification {
                Ok(event) => {
                    // Only process incoming packets, outgoing etc. are not relevant
                    let Event::Incoming(Packet::Publish(p)) = event else {
                        // ConnAck here if reconnecting, need to resubscribe
                        if let Event::Incoming(Packet::ConnAck(_)) = event {
                            println!("Mqtt resubscribing");
                            // Resubscribe
                            // Update
                            mqtt_client
                                .subscribe_all_individual(
                                    &base_settings.update_topic,
                                    &base_settings.pi_zero_id,
                                )
                                .await
                                .unwrap();
                            mqtt_client
                                .subscribe_to_all(&base_settings, &settings)
                                .await
                                .unwrap();
                        }
                        continue;
                    };
                    // Print topic
                    println!("Topic: {:?}", p.topic);
                    // Print payload
                    println!("Received payload: {:?}", &p.payload);

                    if p.topic_matches_pi(&settings.cancel_topic, &base_settings.pi_zero_id) {
                        let task_count = join_set.len();
                        join_set.shutdown().await;
                        let cancelled_tasks = task_count - join_set.len();
                        println!("Cancelled {} tasks", cancelled_tasks);

                        let success_wrapper = SuccessWrapper::success(cancelled_tasks);
                        let json = serde_json::to_string(&success_wrapper).unwrap();

                        mqtt_client
                            .publish_individual(
                                &base_settings.update_topic,
                                &base_settings.pi_zero_id,
                                json,
                            )
                            .await
                            .unwrap_or_default();
                    } else {
                        // Reference counting
                        let base_settings = Arc::clone(&base_settings);
                        let settings = Arc::clone(&settings);
                        let mqtt_client = Arc::clone(&mqtt_client);
                        let http_client = http_client.clone();
                        let should_restart = Arc::clone(&should_restart);
                        let camera_service = Arc::clone(&camera_service);
                        let p = p.clone();
                        // Spawn task
                        join_set.spawn(async move {
                            let mut camera_guard = camera_service.lock().await;

                            handle_notification(
                                &base_settings,
                                &settings,
                                &mqtt_client,
                                &http_client,
                                &should_restart,
                                camera_guard.deref_mut(),
                                &p,
                                wall_nanoseconds,
                            )
                            .await
                        });
                    }
                }
                Err(err) => {
                    println!("CERROR::ConnectionError::{}", err);
                    // Don't send error, as connection error probably means can't send error
                }
            };
        }
    };

    // Handle CTRL+C, otherwise doesn't work
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to listen for event");
        println!("Received CTRL+C, shutting down...");
    };

    tokio::select! {
        _ = mqtt_loop => {},
        _ = ctrl_c => {
            std::process::exit(1);
        },
    }
}
