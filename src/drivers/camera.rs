pub mod camera {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::thread::sleep;
    use std::time::Duration;

    use image::{RgbImage, ImageBuffer};
    use log::*;

    use super::super::light::light;
    use super::super::DeviceError;
    use crate::drivers::hardware_enabled;

    pub fn capture_into<P: AsRef<Path>>(path: P) -> Result<PathBuf, DeviceError> {
        let hardware = hardware_enabled();

        if hardware {
            trace!("Turning light on to capture image");
            light::set(true)?;
            sleep(Duration::from_millis(50));
        }

        let dir_path: PathBuf = path.as_ref().to_path_buf();

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
        let dir = PathBuf::from("./tmp");
        if !dir.exists() {
            std::fs::create_dir(&dir).unwrap();
        }
        assert!(dir.exists());

        let file_path_res = camera::capture_into(&dir);
        assert!(file_path_res.is_ok());
        let file_path = file_path_res.unwrap();
        assert_eq!(file_path.extension().unwrap(), "jpg");

        std::fs::remove_dir_all("./tmp").unwrap();
    }
}
