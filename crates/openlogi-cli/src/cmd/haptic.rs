//! `openlogi haptic` — play named MX Master 4 haptic waveforms through the agent.

use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use clap::Args;
use openlogi_core::device::DeviceInventory;
use openlogi_core::hid::{DeviceRoute, HapticWaveform};
use openlogi_ipc::{AgentClient, AgentSnapshot, InventoryHealth};

use crate::agent;

const INVENTORY_TIMEOUT: Duration = Duration::from_secs(5);
const INVENTORY_RETRY: Duration = Duration::from_millis(50);

#[derive(Debug, Args)]
pub struct HapticArgs {
    /// Waveform name, such as subtle-collision, completed, knock, or happy-alert.
    pub waveform: HapticWaveform,

    /// Case-insensitive substring of the target device name.
    #[arg(long, value_name = "NAME")]
    pub device: Option<String>,
}

pub async fn run(args: HapticArgs) -> Result<()> {
    let client = agent::connect_controller()
        .await
        .context("connect to the OpenLogi Agent; start OpenLogi if it is not running")?;
    let snapshot = wait_for_inventory(&client).await?;
    let candidate = select_device(&snapshot.inventory, args.device.as_deref())?;

    agent::play_haptic(&client, candidate.route, args.waveform)
        .await
        .with_context(|| format!("play {} on {}", args.waveform, candidate.name))?;
    println!("Played {} on {}.", args.waveform, candidate.name);
    Ok(())
}

async fn wait_for_inventory(client: &AgentClient) -> Result<AgentSnapshot> {
    tokio::time::timeout(INVENTORY_TIMEOUT, async {
        loop {
            let snapshot = agent::snapshot(client).await?;
            match snapshot.status.inventory {
                InventoryHealth::Ready => return Ok(snapshot),
                InventoryHealth::Unavailable => {
                    return Err(anyhow!(
                        "the OpenLogi Agent could not enumerate HID++ devices"
                    ));
                }
                InventoryHealth::Scanning => tokio::time::sleep(INVENTORY_RETRY).await,
            }
        }
    })
    .await
    .context("timed out waiting for the OpenLogi Agent to enumerate devices")?
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Candidate {
    route: DeviceRoute,
    name: String,
}

fn select_device(inventories: &[DeviceInventory], query: Option<&str>) -> Result<Candidate> {
    let candidates = inventories
        .iter()
        .flat_map(|inventory| {
            inventory
                .paired
                .iter()
                .filter(|device| {
                    device.online
                        && device
                            .capabilities
                            .is_some_and(|capabilities| capabilities.haptic_feedback)
                })
                .filter_map(|device| {
                    Some(Candidate {
                        route: DeviceRoute::for_slot(inventory, device.slot)?,
                        name: device
                            .codename
                            .clone()
                            .unwrap_or_else(|| format!("Slot {}", device.slot)),
                    })
                })
        })
        .collect::<Vec<_>>();

    let matches = match query {
        Some(query) => {
            let query = query.to_ascii_lowercase();
            candidates
                .into_iter()
                .filter(|candidate| candidate.name.to_ascii_lowercase().contains(&query))
                .collect::<Vec<_>>()
        }
        None => candidates,
    };

    match matches.as_slice() {
        [candidate] => Ok(candidate.clone()),
        [] => match query {
            Some(query) => Err(anyhow!(
                "no online haptic device matches `--device {query}`"
            )),
            None => Err(anyhow!("no online device with haptic feedback found")),
        },
        _ => {
            let names = matches
                .iter()
                .map(|candidate| candidate.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            Err(anyhow!(
                "multiple haptic devices match ({names}); select one with --device"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use openlogi_core::device::{Capabilities, DeviceKind, PairedDevice, ReceiverInfo};

    use super::*;

    fn inventory(name: &str, slot: u8, haptic: bool) -> DeviceInventory {
        DeviceInventory {
            receiver: ReceiverInfo {
                name: "Bolt Receiver".into(),
                vendor_id: 0x046d,
                product_id: 0xc548,
                unique_id: Some(format!("receiver-{slot}")),
            },
            paired: vec![PairedDevice {
                slot,
                codename: Some(name.into()),
                wpid: Some(0xb042),
                kind: DeviceKind::Mouse,
                online: true,
                battery: None,
                model_info: None,
                capabilities: Some(Capabilities {
                    haptic_feedback: haptic,
                    ..Capabilities::default()
                }),
            }],
        }
    }

    #[test]
    fn selection_uses_capability_and_requires_unambiguous_matches() {
        let inventories = vec![
            inventory("MX Master 4 Office", 1, true),
            inventory("MX Master 3S", 2, false),
            inventory("MX Master 4 Travel", 3, true),
        ];

        select_device(&inventories, None)
            .expect_err("multiple capable devices must require explicit selection");
        assert_eq!(
            select_device(&inventories, Some("travel"))
                .expect("specific haptic device")
                .name,
            "MX Master 4 Travel"
        );
        select_device(&inventories, Some("3S"))
            .expect_err("a matching name without haptics must be rejected");
    }
}
