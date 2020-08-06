// Copyright 2018 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use std::rc::Rc;

use crate::{rusqlite, Element, FindSpec, QueryOutput, QueryResults, Rows, Schema};

use query_projector_traits::errors::Result;

use super::Projector;

/// A projector that produces a `QueryResult` containing fixed data.
/// Takes a boxed function that should return an empty result set of the desired type.
pub struct ConstantProjector {
    spec: Rc<FindSpec>,
    results_factory: Box<dyn Fn() -> QueryResults>,
}

impl ConstantProjector {
    pub fn new(
        spec: Rc<FindSpec>,
        results_factory: Box<dyn Fn() -> QueryResults>,
    ) -> ConstantProjector {
        ConstantProjector {
            spec,
            results_factory,
        }
    }

    pub fn project_without_rows(&self) -> Result<QueryOutput> {
        let results = (self.results_factory)();
        let spec = self.spec.clone();
        Ok(QueryOutput { spec, results })
    }
}

// TODO: a ConstantProjector with non-constant pull expressions.

impl Projector for ConstantProjector {
    fn project<'stmt, 's>(
        &self,
        _schema: &Schema,
        _sqlite: &'s rusqlite::Connection,
        _rows: Rows<'stmt>,
    ) -> Result<QueryOutput> {
        self.project_without_rows()
    }

    fn columns<'s>(&'s self) -> Box<dyn Iterator<Item = &Element> + 's> {
        self.spec.columns()
    }
}
