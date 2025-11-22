// Graphical UI using Freya for native desktop GUI functionality.

#[cfg(feature = "gui")]
pub mod freya_ui {
    use freya::prelude::*;
    
    use crate::config::{Config, SpeedUnit};
    use crate::servers::{get_merged_server_list, load_local_server_data};
    use crate::downloader::DownloadResult;
    
    #[derive(Clone, Debug)]
    pub struct TestResult {
        pub server_name: String,
        pub status_code: u16,
        pub bytes_downloaded: u64,
        pub total_time: f64,
        pub connect_time: f64,
        pub ttfb: f64,
        pub speed_mbps: f64,
        pub speed_mb_s: f64,
    }
    
    impl From<(DownloadResult, &str)> for TestResult {
        fn from((result, server_name): (DownloadResult, &str)) -> Self {
            let mbps = (result.bytes_downloaded as f64 * 8.0 / result.total_time) / 1_000_000.0;
            let mb_s = (result.bytes_downloaded as f64 / result.total_time) / 1_000_000.0;
            
            TestResult {
                server_name: server_name.to_string(),
                status_code: result.status_code,
                bytes_downloaded: result.bytes_downloaded,
                total_time: result.total_time,
                connect_time: result.connect_time,
                ttfb: result.ttfb,
                speed_mbps: mbps,
                speed_mb_s: mb_s,
            }
        }
    }
    
    pub fn launch_gui(_config: Config) {
        launch_cfg(
            app,
            LaunchConfig::<()>::new()
                .with_title("Speedo - Network Speed Test")
                .with_size(800.0, 650.0),
        );
    }
    
    #[component]
    fn app() -> Element {
        let config = use_signal(|| crate::config::load_config());
        
        let servers = use_signal(|| {
            let server_data = load_local_server_data();
            get_merged_server_list(&server_data)
        });
        
        let mut selected_server = use_signal(|| 0usize);
        let mut test_running = use_signal(|| false);
        let mut last_result = use_signal(|| None::<TestResult>);
        let mut status_message = use_signal(|| String::from("Ready to test"));
        
        let run_test = move |_| {
            if *test_running.read() {
                return;
            }
            
            let idx = *selected_server.read();
            let servers_list = servers.read();
            
            if let Some(server) = servers_list.get(idx) {
                let server_clone = server.clone();
                let config_clone = config.read().clone();
                let speed_unit = SpeedUnit::from_string(&config_clone.speed_unit);
                
                test_running.set(true);
                status_message.set(format!("Testing {}...", server_clone.name));
                
                spawn(async move {
                    match crate::downloader::download_file_with_progress(
                        &server_clone.url,
                        None,
                        &config_clone.user_agent,
                        speed_unit,
                        false, // Disable progress bar in GUI mode
                    ).await {
                        Ok(result) => {
                            let test_result = TestResult::from((result, server_clone.name.as_str()));
                            last_result.set(Some(test_result.clone()));
                            status_message.set(format!(
                                "Test complete: {:.2} Mbps ({:.2} MB/s)",
                                test_result.speed_mbps,
                                test_result.speed_mb_s
                            ));
                        }
                        Err(e) => {
                            status_message.set(format!("Error: {}", e));
                        }
                    }
                    test_running.set(false);
                });
            }
        };
        
        rsx! {
            rect {
                width: "100%",
                height: "100%",
                background: "rgb(20, 20, 30)",
                padding: "20",
                direction: "vertical",
                
                // Header
                label {
                    color: "rgb(100, 200, 255)",
                    font_size: "36",
                    font_weight: "bold",
                    "Speedo"
                }
                
                label {
                    color: "rgb(150, 150, 160)",
                    font_size: "14",
                    margin: "0 0 20 0",
                    "Network Speed Test Tool"
                }
                
                // Server Selection
                rect {
                    width: "100%",
                    height: "250",
                    direction: "vertical",
                    margin: "0 0 15 0",
                    
                    label {
                        color: "rgb(200, 200, 210)",
                        font_size: "16",
                        margin: "0 0 10 0",
                        "Select Server:"
                    }
                    
                    ScrollView {
                        width: "100%",
                        height: "200",
                        show_scrollbar: true,
                        
                        rect {
                            direction: "vertical",
                            width: "100%",
                            
                            for (idx, server) in servers.read().iter().enumerate() {
                                {
                                    let is_selected = idx == *selected_server.read();
                                    let bg_color = if is_selected {
                                        "rgb(50, 100, 150)"
                                    } else {
                                        "rgb(30, 30, 40)"
                                    };
                                    
                                    let server_name = format!("{} - {}", 
                                        server.name,
                                        server.location.as_ref().unwrap_or(&String::from("Unknown"))
                                    );
                                    
                                    rsx! {
                                        rect {
                                            key: "{idx}",
                                            width: "100%",
                                            height: "40",
                                            background: "{bg_color}",
                                            padding: "10",
                                            margin: "2",
                                            corner_radius: "4",
                                            onclick: move |_| {
                                                selected_server.set(idx);
                                            },
                                            
                                            label {
                                                color: "rgb(220, 220, 230)",
                                                font_size: "14",
                                                "{server_name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Control Button
                rect {
                    width: "100%",
                    height: "50",
                    main_align: "center",
                    margin: "10 0",
                    
                    Button {
                        onpress: run_test,
                        
                        rect {
                            width: "200",
                            height: "45",
                            background: if *test_running.read() {
                                "rgb(100, 100, 110)"
                            } else {
                                "rgb(102, 126, 234)"
                            },
                            corner_radius: "8",
                            main_align: "center",
                            cross_align: "center",
                            
                            label {
                                color: "white",
                                font_size: "16",
                                font_weight: "bold",
                                if *test_running.read() {
                                    "Testing..."
                                } else {
                                    "Run Speed Test"
                                }
                            }
                        }
                    }
                }
                
                // Status Message
                rect {
                    width: "100%",
                    height: "40",
                    background: "rgb(30, 30, 40)",
                    padding: "10",
                    corner_radius: "8",
                    main_align: "center",
                    margin: "10 0",
                    
                    label {
                        color: "rgb(180, 180, 190)",
                        font_size: "14",
                        "{status_message.read()}"
                    }
                }
                
                // Results Display
                if let Some(result) = last_result.read().as_ref() {
                    ScrollView {
                        width: "100%",
                        height: "fill",
                        show_scrollbar: true,
                        
                        rect {
                            width: "100%",
                            direction: "vertical",
                            padding: "15",
                            background: "rgb(25, 35, 45)",
                            corner_radius: "8",
                            
                            label {
                                color: "rgb(100, 200, 255)",
                                font_size: "20",
                                font_weight: "bold",
                                margin: "0 0 15 0",
                                "Test Results"
                            }
                            
                            ResultRow {
                                label: "Server",
                                value: result.server_name.clone()
                            }
                            
                            ResultRow {
                                label: "Status",
                                value: format!("{}", result.status_code)
                            }
                            
                            ResultRow {
                                label: "Downloaded",
                                value: format!("{:.2} MB", result.bytes_downloaded as f64 / 1_000_000.0)
                            }
                            
                            ResultRow {
                                label: "Speed",
                                value: format!("{:.2} Mbps ({:.2} MB/s)", result.speed_mbps, result.speed_mb_s)
                            }
                            
                            ResultRow {
                                label: "Total Time",
                                value: format!("{:.2}s", result.total_time)
                            }
                            
                            ResultRow {
                                label: "Connect Time",
                                value: format!("{:.3}s", result.connect_time)
                            }
                            
                            ResultRow {
                                label: "TTFB",
                                value: format!("{:.3}s", result.ttfb)
                            }
                        }
                    }
                }
            }
        }
    }
    
    #[component]
    fn ResultRow(label: String, value: String) -> Element {
        rsx! {
            rect {
                width: "100%",
                height: "35",
                direction: "horizontal",
                padding: "8 0",
                
                label {
                    color: "rgb(150, 150, 160)",
                    font_size: "14",
                    width: "150",
                    "{label}:"
                }
                
                label {
                    color: "rgb(220, 220, 230)",
                    font_size: "14",
                    "{value}"
                }
            }
        }
    }
}

#[cfg(not(feature = "gui"))]
pub mod freya_ui {
    use crate::config::Config;
    
    pub fn launch_gui(_config: Config) {
        eprintln!("GUI support not compiled. Rebuild with --features gui");
        std::process::exit(1);
    }
}
