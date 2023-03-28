use std::path::{Path, PathBuf};

use image::{ImageBuffer, RgbImage};
use log::*;

use super::DeviceError;

pub struct Camera;

impl Camera {
    pub fn new() -> Self {
        return Self;
    }

    /// Accepts a directory to place the image in. Will fail if the directory doesn't exist
    /// or if the path provided is not a directory. Returns a PathBuf to the file created.
    pub fn capture_and_place<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, DeviceError> {
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
        
        // Generate an image and save it
        let mut img: RgbImage = ImageBuffer::new(50, 50);
        *img.get_pixel_mut(25, 25) = image::Rgb([255, 255, 255]);

        trace!("Writing captured image to {}", img_path.display());
        img.save(&img_path)?;

        // And return the path
        Ok(img_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_and_place_somewhere() {
        let cam = Camera::new();

        let dir = PathBuf::from("./tmp");
        if !dir.exists() {
            std::fs::create_dir(&dir).unwrap();
        }
        assert!(dir.exists());

        let file_path_res = cam.capture_and_place(&dir);
        assert!(file_path_res.is_ok());
        let file_path = file_path_res.unwrap();
        assert_eq!(file_path.extension().unwrap(), "jpg");


        std::fs::remove_dir_all("./tmp").unwrap();
    }
}
