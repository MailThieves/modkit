pub mod camera {
    use std::path::PathBuf;
    use std::process::Command;
    use std::thread::sleep;
    use std::time::Duration;

    use image::{RgbImage, ImageBuffer};
    use log::*;

    use super::super::light::light;
    use super::super::DeviceError;
    use crate::drivers::hardware_enabled;

    const DEFAULT_IMG_DIR: &'static str = "./img";

    pub fn capture_still() -> Result<PathBuf, DeviceError> {
        let hardware = hardware_enabled();

        if hardware {
            trace!("Turning light on to capture image");
            light::set(true)?;
            sleep(Duration::from_millis(50));
        }

        let img_dir = std::env::var("MODKIT_IMG_DIR").unwrap_or(String::from(DEFAULT_IMG_DIR));
        trace!("Using {img_dir} as the image location");

        let dir_path: PathBuf = PathBuf::from(img_dir);

        // Make sure the dir exists
        if !dir_path.exists() {
            error!(
                "Path {} doesn't exist. Please pass a valid directory.",
                dir_path.display()
            );

            return Err(DeviceError::IoError(format!(
                "path {} does not exist",
                dir_path.display()
            )));
        }

        // And make sure it's actually a dir
        if !dir_path.is_dir() {
            error!("Path {} is not a directory", dir_path.display());
            return Err(DeviceError::IoError(format!(
                "path {} is not a directory",
                dir_path.display()
            )));
        }

        // Create a file name within the dir
        let mut img_path = PathBuf::from(dir_path);
        img_path.push(format!("{}.jpg", chrono::Utc::now().timestamp()));

        trace!("File path for captured image: {}", img_path.display());

        if hardware {
            let args = [
                "--drc",
                "high",
                "--width",
                "800",
                "--height",
                "550",
                "--timeout",
                "1",
                "-o",
                &format!("{}", img_path.display()),
            ];

            trace!("Image capture command = `raspistill {:?}`", args);

            trace!("Taking picture with raspistill");
            Command::new("raspistill")
                .args(args)
                .output()
                .expect("Run raspistill command");

            sleep(Duration::from_millis(50));
            trace!("Turning light off after image capture");
            light::set(false)?;
        } else {
            // Generate an image and save it
            let mut img: RgbImage = ImageBuffer::new(50, 50);
            *img.get_pixel_mut(25, 25) = image::Rgb([255, 255, 255]);

            trace!("Writing captured image to {}", img_path.display());
            img.save(&img_path)?;
        }

        // And return the path
        Ok(img_path)
    }
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_capture_and_place_somewhere() {
        let dir = PathBuf::from("./img");
        if !dir.exists() {
            std::fs::create_dir(&dir).unwrap();
        }
        assert!(dir.exists());

        let file_path_res = camera::capture_still();
        assert!(file_path_res.is_ok());
        let file_path = file_path_res.unwrap();
        assert_eq!(file_path.extension().unwrap(), "jpg");

        std::fs::remove_dir_all("./img").unwrap();
    }
}
