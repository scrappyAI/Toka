use anyhow::Result;
use toka_agents::{EventBus, Observation, BaseAgent};

#[tokio::test]
async fn belief_update_moves_probability() -> Result<()> {
    let mut agent = BaseAgent::new("tester");
    let bus = EventBus::new(16);
    agent.set_event_bus(bus);

    // prior absent -> 0.5
    let obs = Observation {
        key: "sky_is_blue".into(),
        evidence_strength: 2.0,
        supports: true,
    };
    agent.observe(obs).await?;
    let prob = agent.beliefs().get("sky_is_blue").unwrap().probability;
    assert!(prob > 0.5);
    Ok(())
}
