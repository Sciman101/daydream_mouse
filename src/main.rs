// Thanks to https://medium.com/hackernoon/how-i-hacked-google-daydream-controller-c4619ef318e4 for a lot of this info
use btleplug::api::{
    Central, Manager as _, Peripheral as _, ScanFilter,
};
use enigo::{
    Button,
    Direction::{Click, Press, Release},
    Enigo, Mouse, Settings,
    {Axis::Horizontal, Axis::Vertical},
    {Coordinate::Abs, Coordinate::Rel},
    Key, Keyboard
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use std::error::Error;
use std::process;
use std::time::Duration;
use futures_util::StreamExt;
use uuid::Uuid;

// Filter for Daydream controller event
const CONTROLLER_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(0x00000001_1000_1000_8000_00805f9b34fb);
use tokio::time;

const MOUSE_SCALE : f64 = 2.0;

async fn find_controller(central: &Adapter) -> Option<Peripheral> {
    for p in central.peripherals().await.unwrap() {
        if p.properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .iter()
            .any(|name| name.contains("Daydream"))
        {
            return Some(p);
        }
    }
    None
}

#[derive(Debug, Default)]
struct DaydreamControllerData {
    touchpad_x: u8,
    touchpad_y: u8,
    app: bool,
    home: bool,
    vol_up: bool,
    vol_down: bool,
    touch_click: bool,
    touch_down: bool,
}

fn parse_raw_controller_data(data: Vec<u8>) -> Option<DaydreamControllerData> {
    // Make sure the vector is of the correct length
    if data.len() != 20 {
        return None;
    }

    Some(DaydreamControllerData {
        touchpad_x: (data[16] << 3) | ((data[17] >> 5) & 0b00011111),
        touchpad_y: (data[17] << 3) | ((data[18] >> 5) & 0b00000111),
        app: (data[18] & 0b00000100) != 0,
        home: (data[18] & 0b00000010) != 0,
        vol_up: (data[18] & 0b00010000) != 0, 
        vol_down: (data[18] & 0b00001000) != 0,
        touch_click: (data[18] & 0b00000001) != 0,
        touch_down: data[17] != 0, // idk if this is 'correct' but it works well enough
    })
}

fn enigo_key_wrapper(enigo : &mut Enigo, state : bool, prev_state : bool, keycode : Key) {
    if state && !prev_state {
        enigo.key(keycode, Press).unwrap();
    }else if state && prev_state {
        enigo.key(keycode, Release).unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let manager = Manager::new().await.unwrap();

    // Get bluetooth adapter
    let central = manager
        .adapters()
        .await
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .nth(0)
        .expect("Unable to find adapters.");
    println!("Adapter found.");
    
    // Scan for devices
    central.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(2)).await;

    // Find the daydream controller
    let controller = find_controller(&central).await.expect("Unable to find Google Daydream controller");
    println!("Controller found.");

    let is_connected = controller.is_connected().await?;
    if !is_connected {
        if let Err(err) = controller.connect().await {
            eprintln!("Error connecting to controller, {}", err);
            process::exit(1);
        }
        println!("Successfully connected to controller1");
    }else{
        println!("Already connected to controller.");
    }

    controller.discover_services().await?;

    let chars = controller.characteristics();
    let notify_characteristic = chars
        .iter()
        .find(|c| c.uuid == CONTROLLER_CHARACTERISTIC_UUID)
        .expect("Unable to find expected characteristic.");
    
    // Subscribe to characteristic
    println!("Subscribing to controller characteristic {:?}", notify_characteristic.uuid);
    controller.subscribe(&notify_characteristic).await?;
    println!("Listening for notification stream");

    // Setup enigo
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    println!("Mouse simulation started.");
    
    // Start dumping data
    let mut notification_stream = controller.notifications().await?;

    // Track previous controller state
    let mut prev_data = DaydreamControllerData::default();

    while let Some(packet) = notification_stream.next().await {
        if let Some(data) = parse_raw_controller_data(packet.value) {
            
            // Touchpad - mouse motion
            if data.touch_down && prev_data.touch_down {
                let mut x_delta = (data.touchpad_x as i32) - (prev_data.touchpad_x as i32);
                let mut y_delta = (data.touchpad_y as i32) - (prev_data.touchpad_y as i32);
                x_delta = ((x_delta as f64) * MOUSE_SCALE) as i32;
                y_delta = ((y_delta as f64) * MOUSE_SCALE) as i32;
                enigo.move_mouse(x_delta, y_delta, Rel).unwrap();
            }

            // Touchpad - left mouse button
            if data.touch_click && !prev_data.touch_click {
                enigo.button(Button::Left, Press).unwrap();
            }else if !data.touch_click && prev_data.touch_click {
                enigo.button(Button::Left, Release).unwrap();
            }

            // App button - right mouse button
            if data.app && !prev_data.app {
                enigo.button(Button::Right, Press).unwrap();
            }else if !data.app && prev_data.app {
                enigo.button(Button::Right, Release).unwrap();
            }

            // Home button - command/windows/super key
            enigo_key_wrapper(&mut enigo, data.home, prev_data.home, Key::Command);
            // Volume control
            enigo_key_wrapper(&mut enigo, data.vol_up, prev_data.vol_up, Key::VolumeUp);
            enigo_key_wrapper(&mut enigo, data.vol_down, prev_data.vol_down, Key::VolumeDown);

            prev_data = data;
        }
    }

    println!("All done.");

    Ok(())
}
