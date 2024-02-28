use std::collections::HashSet;
use std::vec::Drain;
use serde::{Deserialize, Deserializer, Serialize};
use crate::CONFIG_PATH;
use crate::spotify::api::Device;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct DeviceCache {
   device: Option<usize>,
   devices: Vec<Device>,
}

impl DeviceCache {
   pub fn load() -> color_eyre::Result<Self> {
      let devices = std::fs::read_to_string(CONFIG_PATH.join("cache/devices.json"))?;
      Ok(serde_json::from_str(&devices)?)
   }

   pub fn update<I: IntoIterator<Item=Device>>(&mut self, devices: I) {
      for device in devices.into_iter() {
         if !self.devices.contains(&device) {
            self.devices.push(device);
         }
      }
      self.save().unwrap();
   }

   pub fn select(&mut self, index: usize) {
      if index < self.devices.len() {
         self.device = Some(index);
      }
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

   pub fn save(&self) -> color_eyre::Result<()> {
      let path = CONFIG_PATH.join("cache/devices.json");
      if let Some(parent) = path.parent() {
         std::fs::create_dir_all(parent)?;
      }
      std::fs::write(path, serde_json::to_string(self)?.as_bytes())?;
      Ok(())
   }
}