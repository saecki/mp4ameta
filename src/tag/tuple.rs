use crate::{atom, Data, Tag};

/// ### Track
///
/// The track number and total number of tracks are stored in a tuple. If only one is present the
/// other is represented as 0 and will be treated as if nonexistent.
impl Tag {
    /// Returns the track number and the total number of tracks (`trkn`).
    pub fn track(&self) -> (Option<u16>, Option<u16>) {
        let vec = match self.bytes(atom::TRACK_NUMBER).next() {
            Some(v) => v,
            None => return (None, None),
        };

        let track_number = number(vec);
        let total_tracks = total(vec);

        (track_number, total_tracks)
    }

    /// Returns the track number (`trkn`).
    pub fn track_number(&self) -> Option<u16> {
        let vec = self.bytes(atom::TRACK_NUMBER).next()?;

        number(vec)
    }

    /// Returns the total number of tracks (`trkn`).
    pub fn total_tracks(&self) -> Option<u16> {
        let vec = self.bytes(atom::TRACK_NUMBER).next()?;

        total(vec)
    }

    fn set_new_track(&mut self, track_number: u16, total_tracks: u16) {
        let vec = vec![0u16, track_number, total_tracks, 0u16]
            .into_iter()
            .flat_map(|u| u.to_be_bytes().to_vec())
            .collect();

        self.set_data(atom::TRACK_NUMBER, Data::Reserved(vec));
    }

    /// Sets the track number and the total number of tracks (`trkn`).
    pub fn set_track(&mut self, track_number: u16, total_tracks: u16) {
        if let Some(v) = self.bytes_mut(atom::TRACK_NUMBER).next() {
            set_total(v, total_tracks);
            set_number(v, track_number);
            return;
        }

        self.set_new_track(track_number, total_tracks);
    }

    /// Sets the track number (`trkn`).
    pub fn set_track_number(&mut self, track_number: u16) {
        if let Some(v) = self.bytes_mut(atom::TRACK_NUMBER).next() {
            set_number(v, track_number);
            return;
        }

        self.set_new_track(track_number, 0);
    }

    /// Sets the total number of tracks (`trkn`).
    pub fn set_total_tracks(&mut self, total_tracks: u16) {
        if let Some(v) = self.bytes_mut(atom::TRACK_NUMBER).next() {
            set_total(v, total_tracks);
            return;
        }

        self.set_new_track(0, total_tracks);
    }

    /// Removes the track number and the total number of tracks (`trkn`).
    pub fn remove_track(&mut self) {
        self.remove_data(atom::TRACK_NUMBER);
    }

    /// Removes the track number, preserving the total number of tracks if present (`trkn`).
    pub fn remove_track_number(&mut self) {
        if let Some(v) = self.bytes_mut(atom::TRACK_NUMBER).next() {
            if v.len() >= 6 && !(v[4] == 0 && v[5] == 0) {
                v[2] = 0;
                v[3] = 0;
                return;
            }
        }
        self.remove_track();
    }

    /// Removes the total number of tracks, preserving the track number if present (`trkn`).
    pub fn remove_total_tracks(&mut self) {
        if let Some(v) = self.bytes_mut(atom::TRACK_NUMBER).next() {
            if v.len() >= 4 && !(v[2] == 0 && v[3] == 0) {
                v[4] = 0;
                v[5] = 0;
                return;
            }
        }
        self.remove_track();
    }

    /// Returns the track numer and total number of tracks formatted in an easily readable way.
    pub(crate) fn format_track(&self) -> Option<String> {
        match self.track() {
            (Some(d), Some(t)) => Some(format!("track: {} of {}\n", d, t)),
            (Some(d), None) => Some(format!("track: {}\n", d)),
            (None, Some(t)) => Some(format!("track: ? of {}\n", t)),
            (None, None) => None,
        }
    }
}

/// ### Disc
///
/// The disc number and total number of discs are stored in a tuple. If only one is present the
/// other is represented as 0 and will be treated as if nonexistent.
impl Tag {
    /// Returns the disc number and total number of discs (`disk`).
    pub fn disc(&self) -> (Option<u16>, Option<u16>) {
        let vec = match self.bytes(atom::DISC_NUMBER).next() {
            Some(v) => v,
            None => return (None, None),
        };

        let disc_number = number(vec);
        let total_discs = total(vec);

        (disc_number, total_discs)
    }

    /// Returns the disc number (`disk`).
    pub fn disc_number(&self) -> Option<u16> {
        let vec = self.bytes(atom::DISC_NUMBER).next()?;

        number(vec)
    }

    /// Returns the total number of discs (`disk`).
    pub fn total_discs(&self) -> Option<u16> {
        let vec = self.bytes(atom::DISC_NUMBER).next()?;

        total(vec)
    }

    fn set_new_disc(&mut self, disc_number: u16, total_discs: u16) {
        let vec = vec![0u16, disc_number, total_discs]
            .into_iter()
            .flat_map(|u| u.to_be_bytes().to_vec())
            .collect();

        self.set_data(atom::DISC_NUMBER, Data::Reserved(vec));
    }

    /// Sets the disc number and the total number of discs (`disk`).
    pub fn set_disc(&mut self, disc_number: u16, total_discs: u16) {
        if let Some(v) = self.bytes_mut(atom::DISC_NUMBER).next() {
            set_total(v, total_discs);
            set_number(v, disc_number);
            return;
        }

        self.set_new_disc(disc_number, total_discs);
    }

    /// Sets the disc number (`disk`).
    pub fn set_disc_number(&mut self, disc_number: u16) {
        if let Some(v) = self.bytes_mut(atom::DISC_NUMBER).next() {
            set_number(v, disc_number);
            return;
        }

        self.set_new_disc(disc_number, 0);
    }

    /// Sets the total number of discs (`disk`).
    pub fn set_total_discs(&mut self, total_discs: u16) {
        if let Some(v) = self.bytes_mut(atom::DISC_NUMBER).next() {
            set_total(v, total_discs);
            return;
        }

        self.set_new_disc(0, total_discs);
    }

    /// Removes the disc number and the total number of discs (`disk`).
    pub fn remove_disc(&mut self) {
        self.remove_data(atom::DISC_NUMBER);
    }

    /// Removes the disc number, preserving the total number of discs if present (`disk`).
    pub fn remove_disc_number(&mut self) {
        if let Some(v) = self.bytes_mut(atom::DISC_NUMBER).next() {
            if v.len() >= 6 && !(v[4] == 0 && v[5] == 0) {
                v[2] = 0;
                v[3] = 0;
                return;
            }
        }
        self.remove_disc();
    }

    /// Removes the total number of discs, preserving the disc number if present (`disk`).
    pub fn remove_total_discs(&mut self) {
        if let Some(v) = self.bytes_mut(atom::DISC_NUMBER).next() {
            if v.len() >= 4 && !(v[2] == 0 && v[3] == 0) {
                v[4] = 0;
                v[5] = 0;
                return;
            }
        }
        self.remove_disc();
    }

    /// Returns the disc numer and total number of discs formatted in an easily readable way.
    pub(crate) fn format_disc(&self) -> Option<String> {
        match self.disc() {
            (Some(d), Some(t)) => Some(format!("disc: {} of {}\n", d, t)),
            (Some(d), None) => Some(format!("disc: {}\n", d)),
            (None, Some(t)) => Some(format!("disc: ? of {}\n", t)),
            (None, None) => None,
        }
    }
}

fn number(vec: &[u8]) -> Option<u16> {
    be_int!(vec, 2, u16).and_then(|dn| if dn == 0 { None } else { Some(dn) })
}

fn total(vec: &[u8]) -> Option<u16> {
    be_int!(vec, 4, u16).and_then(|dn| if dn == 0 { None } else { Some(dn) })
}

fn set_number(vec: &mut Vec<u8>, number: u16) {
    set_be_int!(vec, 2, number, u16);
}

fn set_total(vec: &mut Vec<u8>, total: u16) {
    set_be_int!(vec, 4, total, u16);
}
