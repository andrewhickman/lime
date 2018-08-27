use std::iter::once;
use std::sync::Arc;

use failure::{err_msg, Fallible};
use vulkano::device::{Device, Queue};
use vulkano::instance::{DeviceExtensions, PhysicalDevice, QueueFamily};

pub struct Context {
    transfer: Arc<Queue>,
    graphics: Arc<Queue>,
    device: Arc<Device>,
}

impl Context {
    pub(crate) fn new(
        phys: PhysicalDevice,
        mut pred: impl FnMut(QueueFamily) -> bool,
        exts: &DeviceExtensions,
    ) -> Fallible<Self> {
        // Required:
        //  - graphics.supports_graphics() && pred(graphics)
        //  - transfer.supports_transfers()
        //
        // Preferred:
        //  - graphics != transfer
        //  - !transfer.supports_graphics()

        let queue_families: Vec<_> = phys
            .queue_families()
            .filter(|fam| fam.supports_transfers())
            .collect();

        let mut graphics = None;
        let mut transfer = queue_families
            .iter()
            .cloned()
            .find(|&fam| !fam.supports_graphics());

        for family in queue_families
            .iter()
            .cloned()
            .filter(|fam| fam.supports_graphics())
        {
            if graphics.is_none() && pred(family) {
                graphics = Some(family);
            } else if transfer.is_none() {
                transfer = Some(family);
            }
        }

        let graphics = graphics.ok_or_else(|| err_msg("no graphics queue found"))?;
        let transfer = transfer.unwrap_or_else(|| graphics.clone());

        let queue_families = [(graphics, 0.4), (transfer, 0.6)];

        let (device, mut queues) = Device::new(
            phys,
            phys.supported_features(),
            exts,
            queue_families.iter().cloned(),
        )?;

        let graphics = queues.next().unwrap();
        let transfer = queues.next().unwrap();

        Ok(Context {
            device,
            graphics,
            transfer,
        })
    }

    pub fn transfer_queue(&self) -> &Arc<Queue> {
        &self.transfer
    }

    pub fn graphics_queue(&self) -> &Arc<Queue> {
        &self.graphics
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn queue_families(&self) -> impl Iterator<Item = QueueFamily> {
        once(self.transfer.family()).chain(once(self.graphics.family()))
    }
}
