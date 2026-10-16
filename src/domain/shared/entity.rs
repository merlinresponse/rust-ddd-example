/// Entity — an object that is defined by WHO it is, not by WHAT its values are.
///
/// DDD role: Entity (an object with an identity that stays the same while its attributes change).
/// Business meaning: two checklist items both titled "Call Bob" are still two different items;
///   ticking one off does not tick off the other. Their *identity* tells them apart.
/// Entity vs Value Object, in one line: compare an Entity by its ID, compare a Value Object by
///   its value.
/// Why modelled this way: entities deliberately do NOT derive `PartialEq`. "Equal fields" is the
///   wrong question for an entity, so we ask `same_identity_as` instead.
/// Pattern: Entity (Eric Evans, Domain-Driven Design, Ch. 5).
pub trait Entity {
    type Id: PartialEq;

    /// The identity: what makes this "the same one" over time.
    fn id(&self) -> &Self::Id;

    /// True when both are the same thing, even if their other fields differ.
    fn same_identity_as(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
