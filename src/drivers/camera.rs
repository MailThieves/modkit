use std::fs::create_dir;
use std::path::{Path, PathBuf};

use image::{ImageBuffer, RgbImage};
use log::*;

use super::DeviceError;

pub struct Camera;

impl Camera {
    pub fn new() -> Self {
        return Self;
    }

    // Simulation for when we're not connected to the camera
    fn capture_img(&self) -> Result<ImageBuffer<image::Rgb<u8>, Vec<u8>>, DeviceError> {
        let mut img: RgbImage = ImageBuffer::new(50, 50);
        *img.get_pixel_mut(25, 25) = image::Rgb([255, 255, 255]);

        Ok(img)
    }

    pub fn capture_default(&self) -> Result<PathBuf, DeviceError> {
        let mut path = PathBuf::from("./img");

        if !path.exists() {
            info!("./img/ directory didn't already exist, I'm creating it for you");
            create_dir(&path)?;
        }
        path.push(format!("{}.jpg", chrono::Utc::now().timestamp()));

        let img = self.capture_img().unwrap();

        trace!("Writing captured image to {}", path.display());
        img.save(&path)?;
        Ok(path)
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
        let img = self.capture_img()?;
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
    fn test_capture_image() {
        let camera = Camera::new();
        let img_path_res = camera.capture_default();

        assert!(img_path_res.is_ok());
        if let Ok(img_path) = img_path_res {
            assert!(img_path.exists());
            assert_eq!(img_path.extension().unwrap(), "jpg");
        }
    }

    #[test]
    fn test_img_dir_doesnt_already_exist() {
        let path = PathBuf::from("./img");
        std::fs::remove_dir_all(&path).unwrap();

        assert!(!path.exists());

        assert!(Camera::new().capture_default().is_ok());
        assert!(path.exists());
    }

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
