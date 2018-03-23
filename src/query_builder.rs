// Copyright 2018 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

#![macro_use]
use mentat_core::{
    Entid,
    NamespacedKeyword,
    TypedValue,
};

use mentat_query::{
    Variable,
};

use edn::{
    DateTime,
    Utc,
    Uuid,
};

use errors::{
    Result,
};

pub enum EntityType {
    Predicate(String),
    Ref(Entid),
    Var(Variable),
}

impl From<String> for EntityType {
    fn from(v: String) -> EntityType {
        EntityType::Predicate(v)
    }
}

impl<'a> From<&'a str> for EntityType {
    fn from(v: &'a str) -> EntityType {
        EntityType::Predicate(v.to_string())
    }
}

impl From<Entid> for EntityType {
    fn from(v: Entid) -> EntityType {
        EntityType::Ref(v)
    }
}

impl From<Variable> for EntityType {
    fn from(v: Variable) -> EntityType {
        EntityType::Var(v)
    }
}

pub enum AttributeType {
    Keyword(NamespacedKeyword),
    Ref(Entid),
    Var(Variable),
}

impl From<NamespacedKeyword> for AttributeType {
    fn from(v: NamespacedKeyword) -> AttributeType {
        AttributeType::Keyword(v)
    }
}

impl From<Entid> for AttributeType {
    fn from(v: Entid) -> AttributeType {
        AttributeType::Ref(v)
    }
}

impl From<Variable> for AttributeType {
    fn from(v: Variable) -> AttributeType {
        AttributeType::Var(v)
    }
}

pub enum QueryValueType {
    Var(Variable),
    Value(TypedValue),
}

impl QueryValueType {
    pub fn value<T>(value: T) -> Self where T: Into<TypedValue> {
        let typed_value: TypedValue = value.into();
        typed_value.into()
    }

    pub fn variable(value: Variable) -> Self {
        value.into()
    }
}

impl From<Variable> for QueryValueType {
    fn from(v: Variable) -> QueryValueType {
        QueryValueType::Var(v)
    }
}

impl From<TypedValue> for QueryValueType {
    fn from(v: TypedValue) -> QueryValueType {
        QueryValueType::Value(v)
    }
}

pub enum FindType {
    Coll,
    Rel,
    Scalar,
    Tuple,
}

#[derive(Default)]
pub struct QueryBuilder {
    find_builder: Option<FindBuilder>,
    where_builder: Option<WhereBuilder>,
    order_builder: Option<OrderBuilder>,
    limit: Option<i32>,
}

pub struct FindBuilder {
    find_type: FindType,
    vars: Vec<Variable>,
}

#[derive(Default)]
pub struct WhereBuilder {
    clauses: Vec<Box<ClauseBuilder>>,
}

pub trait ClauseBuilder {}

#[derive(Default)]
pub struct WhereClauseBuilder {
    entity: Option<EntityType>,
    attribute: Option<AttributeType>,
    value: Option<QueryValueType>,
}

impl ClauseBuilder for WhereClauseBuilder {}

#[derive(Default)]
pub struct OrClauseBuilder {
    clauses: Vec<Box<ClauseBuilder>>,
    join: Option<Variable>,
}

impl ClauseBuilder for OrClauseBuilder {}

#[derive(Default)]
pub struct NotClauseBuilder {
    clauses: Vec<Box<ClauseBuilder>>,
    join: Option<Variable>,
}

impl ClauseBuilder for NotClauseBuilder {}

#[derive(Default)]
pub struct AndClauseBuilder {
    clauses: Vec<Box<ClauseBuilder>>,
}

impl ClauseBuilder for AndClauseBuilder {}

enum QueryOrder {
    Ascending,
    Descending,
}

#[derive(Default)]
pub struct OrderBuilder {
    orders: Vec<(Variable, QueryOrder)>,
}

impl QueryBuilder {
    pub fn add_find<F>(mut self, builder_fn: F) -> Self where F: 'static + FnOnce(FindBuilder) -> FindBuilder {
        self.find_builder = Some(builder_fn(FindBuilder::new()));
        // self.find_builder = Some(builder);
        self
    }

    pub fn add_where<F>(mut self, builder_fn: F) -> Self where F: 'static + FnOnce(WhereBuilder) -> WhereBuilder {
        self.where_builder = Some(builder_fn(WhereBuilder::default()));
        self
    }

    pub fn add_order<F>(mut self, builder_fn: F) -> Self where F: 'static + FnOnce(OrderBuilder) -> OrderBuilder {
        self.order_builder = Some(builder_fn(OrderBuilder::default()));
        self
    }

    pub fn add_limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn execute(self) -> Result<()> {
        println!("executing and destroying self in process");
        Ok(())
    }
}

impl FindBuilder {
    pub fn with_type(find_type: FindType) -> Self {
        FindBuilder {
            find_type: find_type,
            vars: vec![],
        }
    }

    pub fn new() -> Self {
        FindBuilder {
            find_type: FindType::Rel,
            vars: vec![],
        }
    }

    pub fn set_type(mut self, find_type: FindType) -> Self {
        self.find_type = find_type;
        self
    }

    pub fn add<'a, T>(mut self, var: T) -> Self where T: Into<&'a str> {
        self.vars.push(Variable::from_valid_name(var.into()));
        self
    }
}

impl WhereBuilder {
    pub fn add<T>(mut self, clause: T) -> Self where T: 'static + ClauseBuilder {
        self.clauses.push(Box::new(clause));
        self
    }

    pub fn add_clause<F>(mut self, clause_fn: F) -> Self where F: 'static + FnOnce(WhereClauseBuilder) -> WhereClauseBuilder {
        self.clauses.push(Box::new(clause_fn(WhereClauseBuilder::default())));
        self
    }

    pub fn add_or<F>(mut self, or_fn: F) -> Self where F: 'static + FnOnce(OrClauseBuilder) -> OrClauseBuilder {
        self.clauses.push(Box::new(or_fn(OrClauseBuilder::default())));
        self
    }

    pub fn add_not<F>(mut self, not_fn: F) -> Self where F: 'static + FnOnce(NotClauseBuilder) -> NotClauseBuilder {
        self.clauses.push(Box::new(not_fn(NotClauseBuilder::default())));
        self
    }
}

impl WhereClauseBuilder {
    pub fn entity<E>(mut self, entity: E) -> Self where E: Into<EntityType> {
        self.entity = Some(entity.into());
        self
    }

    pub fn attribute<A>(mut self, attribute: A) -> Self where A: Into<AttributeType> {
        self.attribute = Some(attribute.into());
        self
    }

    pub fn value<V>(mut self, value: V) -> Self where V: Into<QueryValueType> {
        self.value = Some(value.into());
        self
    }
}

impl NotClauseBuilder {
    pub fn add<T>(mut self, clause: T) -> Self where T: 'static + ClauseBuilder {
        self.clauses.push(Box::new(clause));
        self
    }

    pub fn join(mut self, var: &str) -> Self {
        self.join = Some(Variable::from_valid_name(var));
        self
    }

    pub fn add_clause<F>(mut self, clause_fn: F) -> Self where F: 'static + FnOnce(WhereClauseBuilder) -> WhereClauseBuilder {
        self.clauses.push(Box::new(clause_fn(WhereClauseBuilder::default())));
        self
    }

    pub fn add_or<F>(mut self, or_fn: F) -> Self where F: 'static + FnOnce(OrClauseBuilder) -> OrClauseBuilder {
        self.clauses.push(Box::new(or_fn(OrClauseBuilder::default())));
        self
    }

    pub fn add_not<F>(mut self, not_fn: F) -> Self where F: 'static + FnOnce(NotClauseBuilder) -> NotClauseBuilder {
        self.clauses.push(Box::new(not_fn(NotClauseBuilder::default())));
        self
    }

    pub fn add_and<F>(mut self, and_fn: F) -> Self where F: 'static + FnOnce(AndClauseBuilder) -> AndClauseBuilder {
        self.clauses.push(Box::new(and_fn(AndClauseBuilder::default())));
        self
    }
}

impl OrClauseBuilder {
    pub fn add<T>(mut self, clause: T) -> Self where T: 'static + ClauseBuilder {
        self.clauses.push(Box::new(clause));
        self
    }

    pub fn join(mut self, var: &str) -> Self {
        self.join = Some(Variable::from_valid_name(var));
        self
    }

    pub fn add_clause<F>(mut self, clause_fn: F) -> Self where F: 'static + FnOnce(WhereClauseBuilder) -> WhereClauseBuilder {
        self.clauses.push(Box::new(clause_fn(WhereClauseBuilder::default())));
        self
    }

    pub fn add_or<F>(mut self, or_fn: F) -> Self where F: 'static + FnOnce(OrClauseBuilder) -> OrClauseBuilder {
        self.clauses.push(Box::new(or_fn(OrClauseBuilder::default())));
        self
    }

    pub fn add_not<F>(mut self, not_fn: F) -> Self where F: 'static + FnOnce(NotClauseBuilder) -> NotClauseBuilder {
        self.clauses.push(Box::new(not_fn(NotClauseBuilder::default())));
        self
    }

    pub fn add_and<F>(mut self, and_fn: F) -> Self where F: 'static + FnOnce(AndClauseBuilder) -> AndClauseBuilder {
        self.clauses.push(Box::new(and_fn(AndClauseBuilder::default())));
        self
    }
}

impl AndClauseBuilder {
    pub fn add<T>(mut self, clause: T) -> Self where T: 'static + ClauseBuilder {
        self.clauses.push(Box::new(clause));
        self
    }

    pub fn add_clause<F>(mut self, clause_fn: F) -> Self where F: 'static + FnOnce(WhereClauseBuilder) -> WhereClauseBuilder {
        self.clauses.push(Box::new(clause_fn(WhereClauseBuilder::default())));
        self
    }

    pub fn add_or<F>(mut self, or_fn: F) -> Self where F: 'static + FnOnce(OrClauseBuilder) -> OrClauseBuilder {
        self.clauses.push(Box::new(or_fn(OrClauseBuilder::default())));
        self
    }

    pub fn add_not<F>(mut self, not_fn: F) -> Self where F: 'static + FnOnce(NotClauseBuilder) -> NotClauseBuilder {
        self.clauses.push(Box::new(not_fn(NotClauseBuilder::default())));
        self
    }

    pub fn add_and<F>(mut self, and_fn: F) -> Self where F: 'static + FnOnce(AndClauseBuilder) -> AndClauseBuilder {
        self.clauses.push(Box::new(and_fn(AndClauseBuilder::default())));
        self
    }
}

impl OrderBuilder {
    pub fn add<'a, T>(self, var: T) -> Self where T: Into<&'a str> {
        self.add_ascending(var)
    }

    pub fn add_ascending<'a, T>(mut self, var: T) -> Self where T: Into<&'a str> {
        self.orders.push((Variable::from_valid_name(var.into()), QueryOrder::Ascending));
        self
    }

    pub fn add_descending<'a, T>(mut self, var: T) -> Self where T: Into<&'a str> {
        self.orders.push((Variable::from_valid_name(var.into()), QueryOrder::Descending));
        self
    }
}

pub fn query() -> QueryBuilder {
    QueryBuilder::default()
}


#[cfg(test)]
mod test {
    extern crate mentat_db;

    use query_builder::*;

    #[test]
    // [:find ?x :where [?x :foo/bar "yyy"]]
    fn test_find_rel() {
        let _ = query().add_find(|find| find.add("?x"))
                              .add_where(|w| {
                                    w.add_clause(|c| c.entity(var!(?x))
                                                      .attribute(kw!(:foo/bar))
                                                      .value(QueryValueType::value("yyy")))
                              }).execute();
        panic!("not complete");
    }

    #[test]
    // [:find ?x :where [?x _ "yyy"]]
    fn test_find_no_attribute() {
        let _ = query().add_find(|find| find.add("?x"))
                              .add_where(|w| {
                                    w.add_clause(|c| c.entity(var!(?x))
                                                      .value(QueryValueType::value("yyy")))
                              }).execute();
        panic!("not complete");
    }

    #[test]
    // [:find ?x . :where [?x :foo/bar "yyy"]]
    fn test_find_scalar() {
        let _ = query().add_find(|find| find.set_type(FindType::Scalar).add("?x"))
                              .add_where(|w| {
                                    w.add_clause(|c| c.entity(var!(?x))
                                                      .attribute(kw!(:foo/bar))
                                                      .value(QueryValueType::value("yyy")))
                              }).execute();
        panic!("not complete");
    }


    // [:find [?url ?description]
    //  :where
    //  (or-join [?page]
    //     [?page :page/url "http://foo.com/"]
    //     [?page :page/title "Foo"])
    // [?page :page/url ?url]
    // [?page :page/description ?description]]
    #[test]
    fn test_find_or_join() {
        let _ = query().add_find(|find| find.add("?url").add("?description"))
                       .add_where(|w| {
                           w.add_or(|or| {
                               or.join("?page")
                                 .add_clause(|c| c.entity(var!(?page))
                                                  .attribute(kw!(:page/url))
                                                  .value(QueryValueType::value("http://foo.com/")))
                                 .add_clause(|c| c.entity(var!(?page))
                                                  .attribute(kw!(:page/title))
                                                  .value(QueryValueType::value("Foo")))
                            })
                            .add_clause(|c| c.entity(var!(?page))
                                             .attribute(kw!(:page/url))
                                             .value(var!(?url)))
                            .add_clause(|c| c.entity(var!(?page))
                                             .attribute(kw!(:page/description))
                                             .value(var!(?description)))
                       })
                       .execute();
        panic!("not complete");
    }


    // [:find ?x :where [?x :foo/baz ?y] :limit 1000]
    #[test]
    fn test_find_with_limit() {
        let _ = query().add_find(|find| find.add("?x"))
                       .add_where(|w| {
                           w.add_clause(|c| c.entity(var!(?x))
                                             .attribute(kw!(:foo/baz))
                                             .value(var!(?y)))
                       })
                       .add_limit(1000)
                       .execute();
        panic!("not complete");
    }

    // [:find ?x :where [?x :foo/baz ?y] :order ?y]
    #[test]
    fn test_find_with_default_order() {
        let _ = query().add_find(|find| find.add("?x"))
                       .add_where(|w| {
                           w.add_clause(|c| c.entity(var!(?x))
                                             .attribute(kw!(:foo/baz))
                                             .value(var!(?y)))
                       })
                       .add_order(|order| order.add("?y"))
                       .execute();
        panic!("not complete");
    }

    // [:find ?x :where [?x :foo/bar ?y] :order (desc ?y)]
    #[test]
    fn test_find_with_desc_order() {
        let _ = query().add_find(|find| find.add("?x"))
                       .add_where(|w| {
                           w.add_clause(|c| c.entity(var!(?x))
                                             .attribute(kw!(:foo/bar))
                                             .value(var!(?y)))
                       })
                       .add_order(|order| order.add_descending("?y"))
                       .execute();
        panic!("not complete");
    }

    // [:find ?x :where [?x :foo/baz ?y] :order (desc ?y) (asc ?x)]
    #[test]
    fn test_find_with_multiple_orders() {
        let _ = query().add_find(|find| find.add("?x"))
                       .add_where(|w| {
                           w.add_clause(|c| c.entity(var!(?x))
                                             .attribute(kw!(:foo/baz))
                                             .value(var!(?y)))
                       })
                       .add_order(|order| order.add_descending("?y").add_ascending("?x"))
                       .execute();
        panic!("not complete");
    }

    // [:find ?x . :where [?x :foo/bar ?y] [(!= ?y 12)]]
    #[test]
    fn test_find_with_predicate() {
        let _ = query().add_find(|find| find.set_type(FindType::Scalar).add("?x"))
                       .add_where(|w| {
                           w.add_clause(|c| c.entity(var!(?x))
                                             .attribute(kw!(:foo/bar))
                                             .value(var!(?y)))
                            .add_clause(|c| c.entity("!=")
                                             .attribute(var!(?y))
                                             .value(QueryValueType::value(12)))
                       })
                       .execute();
        panic!("not complete");
    }

    // figure out ground!
    #[test]
    fn test_find_with_ground() {
        panic!("not complete");
    }

    // figure out fulltext!
    #[test]
    fn test_find_with_fulltext() {
        panic!("not complete");
    }
}
