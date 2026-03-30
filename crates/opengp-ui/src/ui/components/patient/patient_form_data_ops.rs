use super::*;
use opengp_domain::domain::patient::{Address, EmergencyContact, NewPatientData, Patient};

impl PatientForm {
    pub fn from_patient(patient: Patient, theme: Theme) -> Self {
        let gender = patient.gender;
        let concession_type = patient.concession_type;
        let atsi_status = patient.aboriginal_torres_strait_islander;
        let interpreter_required = patient.interpreter_required;

        let mut form = Self::new(theme);
        form.mode = FormMode::Edit(patient.id);

        if let Some(title) = &patient.title {
            form.set_value(FormField::Title, title.clone());
        }
        form.set_value(FormField::FirstName, patient.first_name.clone());
        if let Some(middle_name) = &patient.middle_name {
            form.set_value(FormField::MiddleName, middle_name.clone());
        }
        form.set_value(FormField::LastName, patient.last_name.clone());
        if let Some(preferred_name) = &patient.preferred_name {
            form.set_value(FormField::PreferredName, preferred_name.clone());
        }
        form.set_value(FormField::DateOfBirth, format_date(patient.date_of_birth));
        if let Some(line1) = &patient.address.line1 {
            form.set_value(FormField::AddressLine1, line1.clone());
        }
        if let Some(line2) = &patient.address.line2 {
            form.set_value(FormField::AddressLine2, line2.clone());
        }
        if let Some(suburb) = &patient.address.suburb {
            form.set_value(FormField::Suburb, suburb.clone());
        }
        if let Some(state) = &patient.address.state {
            form.set_value(FormField::State, state.clone());
        }
        if let Some(postcode) = &patient.address.postcode {
            form.set_value(FormField::Postcode, postcode.clone());
        }
        form.set_value(FormField::Country, patient.address.country.clone());
        if let Some(phone_home) = &patient.phone_home {
            form.set_value(FormField::PhoneHome, phone_home.clone());
        }
        if let Some(phone_mobile) = &patient.phone_mobile {
            form.set_value(FormField::PhoneMobile, phone_mobile.clone());
        }
        if let Some(email) = &patient.email {
            form.set_value(FormField::Email, email.clone());
        }
        if let Some(medicare_number) = &patient.medicare_number {
            form.set_value(FormField::MedicareNumber, medicare_number.clone());
        }
        if let Some(medicare_irn) = patient.medicare_irn {
            form.set_value(FormField::MedicareIrn, medicare_irn.to_string());
        }
        if let Some(medicare_expiry) = patient.medicare_expiry {
            form.set_value(FormField::MedicareExpiry, format_date(medicare_expiry));
        }
        if let Some(ihi) = &patient.ihi {
            form.set_value(FormField::Ihi, ihi.clone());
        }
        if let Some(emergency_contact) = &patient.emergency_contact {
            form.set_value(FormField::EmergencyName, emergency_contact.name.clone());
            form.set_value(FormField::EmergencyPhone, emergency_contact.phone.clone());
            form.set_value(
                FormField::EmergencyRelationship,
                emergency_contact.relationship.clone(),
            );
        }
        if let Some(concession_number) = &patient.concession_number {
            form.set_value(FormField::ConcessionNumber, concession_number.clone());
        }
        form.set_value(
            FormField::PreferredLanguage,
            patient.preferred_language.clone(),
        );

        form.data = PatientFormData::from(patient);
        form.set_value(FormField::Gender, gender.to_string());
        if let Some(concession) = concession_type {
            form.set_value(FormField::ConcessionType, concession.to_string());
        }
        if let Some(atsi) = atsi_status {
            form.set_value(FormField::AtsiStatus, atsi.to_string());
        }
        form.set_value(
            FormField::InterpreterRequired,
            if interpreter_required {
                "Yes".to_string()
            } else {
                "No".to_string()
            },
        );

        form
    }

    pub fn to_new_patient_data(&mut self) -> Option<NewPatientData> {
        if !FormNavigation::validate(self) {
            return None;
        }

        let dob = parse_date(&self.get_value(FormField::DateOfBirth))?;
        let gender = self.get_value(FormField::Gender).parse().ok()?;

        let address = Address {
            line1: self.get_value(FormField::AddressLine1).empty_to_none(),
            line2: self.get_value(FormField::AddressLine2).empty_to_none(),
            suburb: self.get_value(FormField::Suburb).empty_to_none(),
            state: self.get_value(FormField::State).empty_to_none(),
            postcode: self.get_value(FormField::Postcode).empty_to_none(),
            country: or_default(self.get_value(FormField::Country), "Australia"),
        };

        let emergency_contact = if !self.get_value(FormField::EmergencyName).is_empty() {
            Some(EmergencyContact {
                name: self.get_value(FormField::EmergencyName),
                phone: self.get_value(FormField::EmergencyPhone),
                relationship: self.get_value(FormField::EmergencyRelationship),
            })
        } else {
            None
        };

        Some(NewPatientData {
            ihi: self.get_value(FormField::Ihi).empty_to_none(),
            medicare_number: self.get_value(FormField::MedicareNumber).empty_to_none(),
            medicare_irn: self.get_value(FormField::MedicareIrn).parse().ok(),
            medicare_expiry: parse_date(&self.get_value(FormField::MedicareExpiry)),
            title: self.get_value(FormField::Title).empty_to_none(),
            first_name: self.get_value(FormField::FirstName),
            middle_name: self.get_value(FormField::MiddleName).empty_to_none(),
            last_name: self.get_value(FormField::LastName),
            preferred_name: self.get_value(FormField::PreferredName).empty_to_none(),
            date_of_birth: dob,
            gender,
            address,
            phone_home: self.get_value(FormField::PhoneHome).empty_to_none(),
            phone_mobile: self.get_value(FormField::PhoneMobile).empty_to_none(),
            email: self.get_value(FormField::Email).empty_to_none(),
            emergency_contact,
            concession_type: self.get_value(FormField::ConcessionType).parse().ok(),
            concession_number: self.get_value(FormField::ConcessionNumber).empty_to_none(),
            preferred_language: Some(self.get_value(FormField::PreferredLanguage)),
            interpreter_required: Some(self.get_value(FormField::InterpreterRequired) == "Yes"),
            aboriginal_torres_strait_islander: self.get_value(FormField::AtsiStatus).parse().ok(),
        })
    }

    pub fn to_update_patient_data(
        &mut self,
    ) -> Option<(Uuid, opengp_domain::domain::patient::UpdatePatientData)> {
        let patient_id = self.patient_id()?;
        if !FormNavigation::validate(self) {
            return None;
        }

        let address = Address {
            line1: self.get_value(FormField::AddressLine1).empty_to_none(),
            line2: self.get_value(FormField::AddressLine2).empty_to_none(),
            suburb: self.get_value(FormField::Suburb).empty_to_none(),
            state: self.get_value(FormField::State).empty_to_none(),
            postcode: self.get_value(FormField::Postcode).empty_to_none(),
            country: or_default(self.get_value(FormField::Country), "Australia"),
        };

        let emergency_contact = if !self.get_value(FormField::EmergencyName).is_empty() {
            Some(EmergencyContact {
                name: self.get_value(FormField::EmergencyName),
                phone: self.get_value(FormField::EmergencyPhone),
                relationship: self.get_value(FormField::EmergencyRelationship),
            })
        } else {
            None
        };

        let data = opengp_domain::domain::patient::UpdatePatientData {
            ihi: self.get_value(FormField::Ihi).empty_to_none(),
            medicare_number: self.get_value(FormField::MedicareNumber).empty_to_none(),
            medicare_irn: self.get_value(FormField::MedicareIrn).parse().ok(),
            medicare_expiry: parse_date(&self.get_value(FormField::MedicareExpiry)),
            title: self.get_value(FormField::Title).empty_to_none(),
            first_name: Some(self.get_value(FormField::FirstName)),
            middle_name: self.get_value(FormField::MiddleName).empty_to_none(),
            last_name: Some(self.get_value(FormField::LastName)),
            preferred_name: self.get_value(FormField::PreferredName).empty_to_none(),
            date_of_birth: parse_date(&self.get_value(FormField::DateOfBirth)),
            gender: self.get_value(FormField::Gender).parse().ok(),
            address: Some(address),
            phone_home: self.get_value(FormField::PhoneHome).empty_to_none(),
            phone_mobile: self.get_value(FormField::PhoneMobile).empty_to_none(),
            email: self.get_value(FormField::Email).empty_to_none(),
            emergency_contact,
            concession_type: self.get_value(FormField::ConcessionType).parse().ok(),
            concession_number: self.get_value(FormField::ConcessionNumber).empty_to_none(),
            preferred_language: Some(self.get_value(FormField::PreferredLanguage)),
            interpreter_required: Some(self.get_value(FormField::InterpreterRequired) == "Yes"),
            aboriginal_torres_strait_islander: self.get_value(FormField::AtsiStatus).parse().ok(),
        };

        Some((patient_id, data))
    }
}
