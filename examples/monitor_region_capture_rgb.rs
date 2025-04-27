use fs_extra::dir;
use std::time::Instant;
use xcap::Monitor;

fn normalized(filename: String) -> String {
    filename.replace(['|', '\\', ':', '/'], "")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let monitors = Monitor::all()?;
    dir::create_all("target/monitors", true).unwrap();

    let monitor = monitors
        .into_iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .expect("No primary monitor found");

    let monitor_width = monitor.width()?;
    let monitor_height = monitor.height()?;

    let region_width = 400u32;
    let region_height = 300u32;

    let x = ((monitor_width as i32) - (region_width as i32)) / 2;
    let y = ((monitor_height as i32) - (region_height as i32)) / 2;

    let start = Instant::now();

    let image_rgb = monitor.capture_region_rgb(x, y, region_width, region_height)?;
    let end = start.elapsed();
    println!(
        "Time to record region as rgb of size {}x{}: {:?}",
        image_rgb.width(),
        image_rgb.height(),
        end
    );

    let start = Instant::now();

    let image_rgba = monitor.capture_region(x, y, region_width, region_height)?;
    let end = start.elapsed();
    println!(
        "Time to record region as rgba of size {}x{}: {:?}",
        image_rgba.width(),
        image_rgba.height(),
        end
    );

    // image_rgb
    //     .save(format!(
    //         "target/monitors/monitor-{}-region-rgb.png",
    //         normalized(monitor.name().unwrap())
    //     ))
    //     .unwrap();

    // image_rgba
    //     .save(format!(
    //         "target/monitors/monitor-{}-region-rgba.png",
    //         normalized(monitor.name().unwrap())
    //     ))
    //     .unwrap();

    Ok(())
}
