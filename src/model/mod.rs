mod organized;
mod import;

pub struct Artist {

}

pub struct ReleaseGroup {
    pub title: String,
    pub artist_credit: ArtistCredit,
    pub releases: Vec<Release>,
}

pub struct Release {
    pub title: String,
    pub artist_credit: ArtistCredit,
    pub release_group: ReleaseGroup,
}

pub struct Track {
    pub title: String,
    pub length: u32,
    pub artist_credit: ArtistCredit,
}

pub struct ArtistCredit {
    pub artists: Vec<Artist>,
}

pub struct Diff {
    
}
