use std::{fmt::Debug, sync::Arc};

use dashmap::DashMap;
use tea_actorx_core::{ActorId, InstanceId, RegId};
use tea_actorx_signer::Metadata;
use tea_codec::ArcIterExt;

use crate::{
	actor::{looped::ActorFactory, ActorAgent, ActorContext},
	error::Result,
	ActorHostRef,
};

#[derive(Debug, Clone)]
pub struct Registry {
	id: RegId,
	state: Arc<State>,
}

struct State {
	factory: Box<dyn ActorFactory>,
	instances: DashMap<InstanceId, Instance>,
}

impl Debug for State {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("State")
			.field("instances", &self.instances)
			.finish()
	}
}

#[derive(Debug, Clone)]
struct Instance {
	actor: ActorAgent,
}

impl Registry {
	pub fn new(factory: Box<dyn ActorFactory>, id: RegId) -> Self {
		Self {
			id,
			state: Arc::new(State {
				factory,
				instances: DashMap::new(),
			}),
		}
	}

	pub fn id(&self) -> &RegId {
		&self.id
	}

	pub fn actors(&self) -> impl Iterator<Item = ActorAgent> + Send + Sync {
		let it = self.state.arc_iter::<_, DashMap<_, _>>(|x| &x.instances);
		it.map(|x| x.actor)
	}

	pub async fn actor(&self, inst: &InstanceId, host: ActorHostRef) -> Result<ActorAgent> {
		let actor = match self.state.instances.entry(inst.clone()) {
			dashmap::mapref::entry::Entry::Occupied(x) => x.get().actor.clone(),
			dashmap::mapref::entry::Entry::Vacant(x) => {
				let actor = self
					.state
					.factory
					.create(ActorContext {
						host,
						id: ActorId {
							reg: self.id.clone(),
							inst: inst.clone(),
						},
					})
					.await?;
				x.insert(Instance {
					actor: actor.clone(),
				});
				actor
			}
		};
		Ok(actor)
	}

	pub fn metadata(&self) -> Option<&Arc<Metadata>> {
		self.state.factory.metadata()
	}

	pub fn drop_actor(&self, id: &InstanceId) {
		self.state.instances.remove(id);
	}
}
