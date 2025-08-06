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
use crate::utils::AsyncClientExt;
use rumqttc::v5::mqttbytes::v5::Packet;
use rumqttc::v5::Event;
use std::env;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use nix::sys::time::TimeValLike;
use tokio::signal;
use tokio::sync::Mutex;

// const VERSION: &str = concat!("MYAPP_VERSION=", env!("CARGO_PKG_VERSION"));
pub const MYAPPVERSION: &'static str = concat!("[MYAPPVERSION:", env!("CARGO_PKG_VERSION"), "]");

#[tokio::main]
async fn main() {
    println!("Version: {}", MYAPPVERSION);
    // Critical startup
    let (base_settings, mqtt_client, mut mqtt_eventloop, http_client, current_exe) =
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
        loop {
            // Restart, if needed
            if should_restart.load(Ordering::Relaxed) {
                let current_exe = Arc::clone(&current_exe);
                restart(&current_exe);
                break;
            }

            // Wait for message
            let notification = mqtt_eventloop.poll().await;
            let wall_nanoseconds = nix::time::clock_gettime(nix::time::ClockId::CLOCK_REALTIME).ok().map(|wall_time| wall_time.num_nanoseconds());

            // Reference counting
            let base_settings = Arc::clone(&base_settings);
            let settings = Arc::clone(&settings);
            let mqtt_client = Arc::clone(&mqtt_client);
            let http_client = http_client.clone();
            let should_restart = Arc::clone(&should_restart);
            let camera_service = Arc::clone(&camera_service);

            // Process in task to start receiving next message asap
            tokio::task::spawn(async move {
                match notification {
                    Ok(event) => {
                        // Only process incoming packets, outgoing etc. are not relevant
                        let Event::Incoming(Packet::Publish(p)) = &event else {
                            // TODO: Maybe handle subscriptions somewhere else to reuse code
                            // ConnAck here if reconnecting, need to resubscribe
                            if let Event::Incoming(Packet::ConnAck(_)) = event {
                                // Resubscribe
                                // Update
                                mqtt_client
                                    .subscribe_all_individual(&base_settings.update_topic, &base_settings.pi_zero_id)
                                    .await
                                    .unwrap();
                                // NTP
                                mqtt_client
                                    .subscribe_all_individual(
                                        settings.ntp_topic.as_str(),
                                        base_settings.pi_zero_id.as_str(),
                                    )
                                    .await
                                    .unwrap();
                                // Taking pictures
                                mqtt_client
                                    .subscribe_all_individual(
                                        settings.camera_topic.as_str(),
                                        base_settings.pi_zero_id.as_str(),
                                    )
                                    .await
                                    .unwrap();
                                // Linux commands
                                mqtt_client
                                    .subscribe_all_individual(
                                        settings.command_topic.as_str(),
                                        base_settings.pi_zero_id.as_str(),
                                    )
                                    .await
                                    .unwrap();
                                // Status
                                mqtt_client
                                    .subscribe_all_individual(
                                        settings.status_topic.as_str(),
                                        base_settings.pi_zero_id.as_str(),
                                    )
                                    .await
                                    .unwrap();
                            }
                            return;
                        };
                        // Print topic
                        println!("Topic: {:?}", p.topic);
                        // Extract payload, print it
                        // let payload = str::from_utf8(&p.payload).unwrap();
                        println!("Received payload: {:?}", &p.payload);

                        handle_notification(
                            &base_settings,
                            &settings,
                            &mqtt_client,
                            &http_client,
                            &should_restart,
                            camera_service.lock().await.deref_mut(),
                            &p,
                            wall_nanoseconds
                        )
                        .await
                    }
                    Err(err) => {
                        println!("CERROR::ConnectionError::{}", err);
                        // Don't send error, as connection error probably means can't send error
                    }
                };
            });
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
