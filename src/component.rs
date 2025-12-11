use crate::{reference::Reference, spline::Spline, vector::*};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Peak {
    pub start: Point2D,
    pub retention_point: Point2D,
    pub end: Point2D,
    pub height: f64,
    pub area: f64,
}

impl Peak {
    pub fn get_retention_location(&self, spline: Option<&Spline>) -> Option<f64> {
        match spline {
            None => Some(self.retention_point.x()),
            Some(spline) => spline.evaluate(self.retention_point.x()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Component {
    Unknown(Peak),
    Located(Peak, Reference),
    Reference(Reference),
}

impl Component {
    pub fn get_experimental_location(&self, spline: Option<&Spline>) -> Option<f64> {
        match self {
            Component::Unknown(peak) => peak.get_retention_location(spline),
            Component::Located(peak, _) => peak.get_retention_location(spline),
            Component::Reference(_) => None,
        }
    }

    pub fn get_expected_location(&self, spline: Option<&Spline>) -> Option<f64> {
        match self {
            Component::Unknown(_) => None,
            Component::Located(_, reference) => reference.get_expected_location(spline),
            Component::Reference(reference) => reference.get_expected_location(spline),
        }
    }

    pub fn get_experimental_rt(&self) -> Option<f64> {
        match self {
            Component::Unknown(peak) => Some(peak.retention_point.x()),
            Component::Located(peak, _) => Some(peak.retention_point.x()),
            Component::Reference(_) => None,
        }
    }

    pub fn get_expected_rt(&self) -> Option<f64> {
        match self {
            Component::Unknown(_) => None,
            Component::Located(_, reference) => reference.get_expected_rt(),
            Component::Reference(reference) => reference.get_expected_rt(),
        }
    }

    pub fn get_experimental_gu(&self, maybe_spline: Option<&Spline>) -> Option<f64> {
        match self {
            Component::Unknown(peak) => {
                maybe_spline.map_or(None, |spline| spline.evaluate(peak.retention_point.x()))
            }
            Component::Located(peak, _) => {
                maybe_spline.map_or(None, |spline| spline.evaluate(peak.retention_point.x()))
            }
            Component::Reference(_) => None,
        }
    }

    pub fn get_expected_gu(&self, spline: Option<&Spline>) -> Option<f64> {
        match self {
            Component::Unknown(_) => None,
            Component::Located(_, reference) => reference.get_expected_gu(spline),
            Component::Reference(reference) => reference.get_expected_gu(spline),
        }
    }

    pub fn get_area(&self) -> Option<f64> {
        match self {
            Component::Unknown(peak) => Some(peak.area),
            Component::Located(peak, _) => Some(peak.area),
            Component::Reference(_) => None,
        }
    }

    pub fn point_label(&self) -> String {
        //TODO: implement GU
        match &self {
            Component::Unknown(peak) => {
                format!("[Unknown, {:.3}]", peak.retention_point.x())
            }
            Component::Located(peak, reference) => {
                if let Some(name) = &reference.name {
                    format!("[{}, {:.3}]", name, peak.retention_point.x())
                } else {
                    format!("[Unnamed, {:.3}]", peak.retention_point.x())
                }
            }
            Component::Reference(_) => {
                format!("67")
            }
        }
    }
}
