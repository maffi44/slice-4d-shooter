#[derive(Clone)]
pub enum BonusSpotStatus
{
    BonusOnTheSpot,
    BonusCollected(
        // ActorID of a player collected the bonus
        u128
    )
}

impl From<client_server_protocol::BonusSpotStatus> for BonusSpotStatus
{
    fn from(value: client_server_protocol::BonusSpotStatus) -> Self {
        match value
        {
            client_server_protocol::BonusSpotStatus::BonusOnTheSpot =>
            {
                BonusSpotStatus::BonusOnTheSpot
            }
            client_server_protocol::BonusSpotStatus::BonusCollected(id) =>
            {
                BonusSpotStatus::BonusCollected(id)
            }
        }
    }
}