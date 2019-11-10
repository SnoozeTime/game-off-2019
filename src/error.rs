use snafu::Snafu;

pub type GameResult<T> = std::result::Result<T, GameError>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum GameError {
    #[snafu(display("Bullet with index {} was not found.", idx))]
    BulletNotFound { idx: usize },
}
