mod engine;
mod handler;

use std::path::Path;
use std::sync::Arc;

use bytes::Bytes;
use tokio::time::{sleep, Duration};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ricq::client::{Connector as _, DefaultConnector, Token};
use ricq::ext::common::after_login;
use ricq::{Client, Device, Protocol};
use ricq::{LoginResponse, QRCodeConfirmed, QRCodeImageFetch, QRCodeState};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("ricq", Level::DEBUG)
                .with_target("boar_bot", Level::DEBUG),
        )
        .init();

    let device = match Path::new("device.json").exists() {
        true => serde_json::from_str(
            &tokio::fs::read_to_string("device.json")
                .await
                .expect("failed to read device.json"),
        )
            .expect("failed to parse device info"),
        false => {
            let d = Device::random();
            tokio::fs::write("device.json", serde_json::to_string(&d).unwrap())
                .await
                .expect("failed to write device info to file");
            d
        }
    };

    // register modules here
    let client = Arc::new(Client::new(
        device,
        Protocol::AndroidWatch.into(),
        engine::Engine::default()
            .register_module(handler::LoggingHandler)
            .register_module(handler::EchoHandler),
    ));

    let handle = tokio::spawn({
        let client = client.clone();
        let stream = DefaultConnector.connect(&client).await.unwrap();
        async move { client.start(stream).await }
    });
    tokio::task::yield_now().await;

    match Path::new("session.token").exists() {
        true => {
            // try recovery from token if session.token exists
            let token = tokio::fs::read_to_string("session.token")
                .await
                .expect("failed to read token");
            let token: Token = serde_json::from_str(&token).expect("failed to parse token");
            client
                .token_login(token)
                .await
                .expect("failed to login with token");
        }
        false => {
            // otherwise, do qrcode login
            let mut resp = client.fetch_qrcode().await.expect("failed to fetch qrcode");

            let mut image_sig = Bytes::new();
            loop {
                match resp {
                    QRCodeState::ImageFetch(QRCodeImageFetch {
                                                ref image_data,
                                                ref sig,
                                            }) => {
                        tokio::fs::write("qrcode.png", &image_data)
                            .await
                            .expect("failed to write file");
                        image_sig = sig.clone();
                        tracing::info!("二维码: qrcode.png");
                    }
                    QRCodeState::WaitingForScan => {
                        tracing::info!("二维码待扫描")
                    }
                    QRCodeState::WaitingForConfirm => {
                        tracing::info!("二维码待确认")
                    }
                    QRCodeState::Timeout => {
                        tracing::info!("二维码已超时，重新获取");
                        if let QRCodeState::ImageFetch(QRCodeImageFetch {
                                                           ref image_data,
                                                           ref sig,
                                                       }) = client.fetch_qrcode().await.expect("failed to fetch qrcode")
                        {
                            tokio::fs::write("qrcode.png", &image_data)
                                .await
                                .expect("failed to write file");
                            image_sig = sig.clone();
                            tracing::info!("二维码: qrcode.png");
                        }
                    }
                    QRCodeState::Confirmed(QRCodeConfirmed {
                                               ref tmp_pwd,
                                               ref tmp_no_pic_sig,
                                               ref tgt_qr,
                                               ..
                                           }) => {
                        tracing::info!("二维码已确认");
                        let mut login_resp = client
                            .qrcode_login(tmp_pwd, tmp_no_pic_sig, tgt_qr)
                            .await
                            .expect("failed to qrcode login");
                        if let LoginResponse::DeviceLockLogin { .. } = login_resp {
                            login_resp = client
                                .device_lock_login()
                                .await
                                .expect("failed to device lock login");
                        }
                        tracing::info!("{:?}", login_resp);
                        break;
                    }
                    QRCodeState::Canceled => {
                        panic!("二维码已取消")
                    }
                }
                sleep(Duration::from_secs(5)).await;
                resp = client
                    .query_qrcode_result(&image_sig)
                    .await
                    .expect("failed to query qrcode result");
            }
        }
    };

    after_login(&client).await;
    {
        // write token to file
        std::fs::write(
            "session.token",
            serde_json::to_string_pretty(&client.gen_token().await).unwrap(),
        ).unwrap();

        tracing::info!("{:?}", client.get_friend_list().await);
        tracing::info!("{:?}", client.get_group_list().await);
    }

    handle.await.unwrap();
}