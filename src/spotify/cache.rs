use std::path::PathBuf;

use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize};
use serde::ser::SerializeMap;

use crate::CONFIG_PATH;
use crate::spotify::response::Device;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct DeviceCache {
   pub device: Option<usize>,
   pub devices: Vec<Device>,
}

lazy_static! {
   static ref DEVICE_CACHE: PathBuf = CONFIG_PATH.join("cache/devices.json");
}

impl DeviceCache {
   pub fn load() -> color_eyre::Result<Self> {
      let devices = std::fs::read_to_string(DEVICE_CACHE.as_path())?;
      Ok(serde_json::from_str(&devices)?)
   }

   pub fn update<I: IntoIterator<Item=Device>>(&mut self, devices: I) {
      for device in devices.into_iter() {
         if !self.devices.contains(&device) {
            self.devices.push(device);
         }
      }
      // self.save();
   }

   pub fn select(&mut self, index: usize) {
      if index < self.devices.len() {
         self.device = Some(index);
         // self.save()
      }
   }

   pub fn set_device(&mut self, device: Device) {
      // If device isn't in cached devices add it
      if !self.devices.contains(&device) {
         self.devices.push(device.clone());
         // self.save();
      }

      // Get position of device and set it as selected device
      let index = self.devices.iter().position(|d| d == &device).unwrap();
      self.select(index);
   }

   pub fn device(&self) -> Option<&Device> {
      match self.device {
         Some(device) => self.devices.get(device),
         None => None
      }
   }

   /// Reference to all devices. Order is not gauaranteed
   pub fn devices(&self) -> &Vec<Device> {
      &self.devices
   }

   pub fn drain(self) -> Vec<Device> {
      self.devices
   }

   fn save(&self) {
      if let Some(parent) = DEVICE_CACHE.parent() {
         std::fs::create_dir_all(parent).unwrap();
      }
      std::fs::write(DEVICE_CACHE.as_path(), serde_json::to_string(self).unwrap().as_bytes()).unwrap();
   }
}