//! GraphSON V3 [docs](http://tinkerpop.apache.org/docs/current/dev/io/)
//!

use crate::structure::{
    Edge, GValue, IntermediateRepr, List, Map, Metric, Path, Property, TraversalExplanation,
    TraversalMetrics, Vertex, VertexProperty, GID,
};
use crate::GremlinError;
use crate::GremlinResult;
use chrono::offset::TimeZone;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;

static G_METRICS: &'static str = "g:Metrics";
static G_TRAVERSAL_EXPLANATION: &'static str = "g:TraversalExplanation";
static G_TRAVERSAL_METRICS: &'static str = "g:TraversalMetrics";

// Deserialize a JSON value to a GID
pub fn deserialize_id<T>(reader: &T, val: &Value) -> GremlinResult<GID>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    match reader(val) {
        Ok(result) => match result {
            GValue::String(d) => Ok(GID::String(d)),
            GValue::Int32(d) => Ok(GID::Int32(d)),
            GValue::Int64(d) => Ok(GID::Int64(d)),
            _ => Err(GremlinError::Json(format!("{} cannot be an id", val))),
        },
        Err(e) => match e {
            GremlinError::Json(_e) => Ok(GID::String(val.to_string())),
            _ => Err(e),
        },
    }
}

// Date deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_date_2)
pub fn deserialize_date<T>(_: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let val = expect_i64!(val);
    Ok(GValue::from(Utc.timestamp(val, 0)))
}

// Long deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_long_2)
pub fn deserialize_g64<T>(_: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let val = expect_i64!(val);
    Ok(GValue::from(val))
}

// UUID deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_uuid_2)
pub fn deserialize_uuid<T>(_: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let val = get_value!(val, Value::String)?;
    let uuid = uuid::Uuid::parse_str(&val)?;
    Ok(GValue::Uuid(uuid))
}

// Integer deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_integer_2)
pub fn deserialize_g32<T>(_: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let val = expect_i32!(val);
    Ok(GValue::from(val))
}

// Float deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_float_2)
pub fn deserialize_f32<T>(_: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let val = expect_float!(val);
    Ok(GValue::from(val))
}
// Double deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_float_2)
pub fn deserialize_f64<T>(_: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let val = expect_double!(val);
    Ok(GValue::from(val))
}

// List deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_list)
pub fn deserialize_list<T>(reader: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let val = get_value!(val, Value::Array)?;
    let mut elements = Vec::with_capacity(val.len());
    for item in val {
        elements.push(reader(item)?)
    }
    Ok(elements.into())
}

// Map deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_map)
pub fn deserialize_map<T>(reader: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let val = get_value!(val, Value::Array)?;
    let mut map = HashMap::new();
    if !val.is_empty() {
        let mut x = 0;
        while x < val.len() {
            let key = get_value!(&val[x], Value::String)?;
            let value = reader(&val[x + 1])?;
            map.insert(key.clone(), value);
            x += 2;
        }
    }
    Ok(map.into())
}

// Vertex deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_vertex_3)
pub fn deserialize_vertex<T>(reader: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let label = val
        .get("label")
        .map(|f| get_value!(f, Value::String).map(Clone::clone))
        .unwrap_or_else(|| Ok(String::from("vertex")))?;

    let id = deserialize_id(reader, &val["id"])?;

    Ok(Vertex::new(
        id,
        label,
        deserialize_vertex_properties(reader, &val["properties"])?,
    )
    .into())
}

// Edge deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_edge_3)
pub fn deserialize_edge<T>(reader: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let label = val
        .get("label")
        .map(|f| get_value!(f, Value::String).map(Clone::clone))
        .unwrap_or_else(|| Ok(String::from("edge")))?;

    let id = deserialize_id(reader, &val["id"])?;

    let in_v_id = deserialize_id(reader, &val["inV"])?;
    let in_v_label = get_value!(&val["inVLabel"], Value::String)?.clone();

    let out_v_id = deserialize_id(reader, &val["outV"])?;
    let out_v_label = get_value!(&val["outVLabel"], Value::String)?.clone();

    Ok(Edge::new(
        id,
        label,
        in_v_id,
        in_v_label,
        out_v_id,
        out_v_label,
        HashMap::new(),
    )
    .into())
}

// Path deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_path_3)
pub fn deserialize_path<T>(reader: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let labels = reader(&val["labels"])?;

    let objects = reader(&val["objects"])?.take::<List>()?;

    Ok(Path::new(labels, objects).into())
}

// Traversal Metrics deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_traversalmetrics)
pub fn deserialize_metrics<T>(reader: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let mut metrics = reader(&val)?.take::<Map>()?;

    let duration = get_and_remove(&mut metrics, "dur", G_TRAVERSAL_METRICS)?.take::<f64>()?;

    let m = get_and_remove(&mut metrics, "metrics", G_TRAVERSAL_METRICS)?
        .take::<List>()?
        .drain(0..)
        .map(|e| e.take::<Metric>())
        .filter_map(Result::ok)
        .collect();

    Ok(TraversalMetrics::new(duration, m).into())
}

// Metrics deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_metrics)
pub fn deserialize_metric<T>(reader: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let mut metric = reader(&val)?.take::<Map>()?;

    let duration = get_and_remove(&mut metric, "dur", G_METRICS)?.take::<f64>()?;
    let id = get_and_remove(&mut metric, "id", G_METRICS)?.take::<String>()?;
    let name = get_and_remove(&mut metric, "name", G_METRICS)?.take::<String>()?;

    let mut counts = get_and_remove(&mut metric, "counts", G_METRICS)?.take::<Map>()?;
    let traversers = get_and_remove(&mut counts, "traverserCount", G_METRICS)?.take::<i64>()?;
    let count = get_and_remove(&mut counts, "elementCount", G_METRICS)?.take::<i64>()?;

    let mut annotations = get_and_remove(&mut metric, "annotations", G_METRICS)?.take::<Map>()?;
    let perc_duration = get_and_remove(&mut annotations, "percentDur", G_METRICS)?.take::<f64>()?;

    Ok(Metric::new(id, name, duration, count, traversers, perc_duration).into())
}

pub fn deserialize_explain<T>(reader: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let mut explain = reader(&val)?.take::<Map>()?;

    let original = get_and_remove(&mut explain, "original", G_TRAVERSAL_EXPLANATION)?
        .take::<List>()?
        .drain(0..)
        .map(|s| s.take::<String>())
        .filter_map(Result::ok)
        .collect();

    let finals = get_and_remove(&mut explain, "final", G_TRAVERSAL_EXPLANATION)?
        .take::<List>()?
        .drain(0..)
        .map(|s| s.take::<String>())
        .filter_map(Result::ok)
        .collect();

    let intermediate = get_and_remove(&mut explain, "intermediate", G_TRAVERSAL_EXPLANATION)?
        .take::<List>()?
        .drain(0..)
        .map(|s| s.take::<Map>())
        .filter_map(Result::ok)
        .map(map_intermediate)
        .filter_map(Result::ok)
        .collect();

    Ok(TraversalExplanation::new(original, finals, intermediate).into())
}

fn map_intermediate(mut m: Map) -> GremlinResult<IntermediateRepr> {
    let traversal = get_and_remove(&mut m, "traversal", G_TRAVERSAL_EXPLANATION)?
        .take::<List>()?
        .drain(0..)
        .map(|s| s.take::<String>())
        .filter_map(Result::ok)
        .collect();

    let strategy = get_and_remove(&mut m, "strategy", G_TRAVERSAL_EXPLANATION)?.take::<String>()?;

    let category = get_and_remove(&mut m, "category", G_TRAVERSAL_EXPLANATION)?.take::<String>()?;

    Ok(IntermediateRepr::new(traversal, strategy, category))
}

// Vertex Property deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_vertexproperty_3)
pub fn deserialize_vertex_property<T>(reader: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let label = val
        .get("label")
        .map(|f| get_value!(f, Value::String).map(Clone::clone))
        .unwrap_or_else(|| Ok(String::from("vertex_property")))?;

    let id = deserialize_id(reader, &val["id"])?;
    let v = reader(&val["value"])?;
    Ok(VertexProperty::new(id, label, v).into())
}

// Property deserializer [docs](http://tinkerpop.apache.org/docs/current/dev/io/#_property_3)
pub fn deserialize_property<T>(reader: &T, val: &Value) -> GremlinResult<GValue>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    let label = val
        .get("key")
        .map(|f| get_value!(f, Value::String).map(Clone::clone))
        .unwrap_or_else(|| Ok(String::from("property")))?;

    let v = reader(&val["value"])?;
    Ok(Property::new(label, v).into())
}

// deserialzer v3
g_serielizer!(deserializer_v3, {
    "g:Int32" => deserialize_g32,
    "g:Int64" => deserialize_g64,
    "g:Float" => deserialize_f32,
    "g:Double" => deserialize_f64,
    "g:Date" => deserialize_date,
    "g:UUID" => deserialize_uuid,
    "g:List" => deserialize_list,
    "g:Set" => deserialize_list,
    "g:Map" => deserialize_map,
    "g:Vertex" => deserialize_vertex,
    "g:VertexProperty" => deserialize_vertex_property,
    "g:Property" => deserialize_property,
    "g:Edge" => deserialize_edge,
    "g:Path" => deserialize_path,
    "g:TraversalMetrics" => deserialize_metrics,
    "g:Metrics" => deserialize_metric,
    "g:TraversalExplanation" => deserialize_explain
});

fn deserialize_vertex_properties<T>(
    reader: &T,
    properties: &Value,
) -> GremlinResult<HashMap<String, Vec<VertexProperty>>>
where
    T: Fn(&Value) -> GremlinResult<GValue>,
{
    match properties {
        Value::Object(o) => {
            let mut p = HashMap::new();
            for (k, v) in o {
                match v {
                    Value::Array(arr) => {
                        let mut vec = vec![];
                        for elem in arr {
                            vec.push(reader(elem)?.take()?);
                        }
                        p.insert(k.clone(), vec);
                    }
                    _ => {
                        return Err(GremlinError::Json(format!(
                            "Expected object or null for properties. Found {}",
                            properties
                        )));
                    }
                };
            }
            Ok(p)
        }

        Value::Null => Ok(HashMap::new()),
        _ => Err(GremlinError::Json(format!(
            "Expected object or null for properties. Found {}",
            properties
        ))),
    }
}

fn get_and_remove(map: &mut Map, field: &str, owner: &str) -> GremlinResult<GValue> {
    map.remove(field)
        .ok_or_else(|| GremlinError::Json(format!("Field {} not found in {}", field, owner)))
}
// TESTS
#[cfg(test)]
mod tests {

    use super::deserializer_v3;
    use serde_json::json;

    use crate::{edge, vertex};

    use crate::structure::{
        GValue, Metric, Path, Property, TraversalMetrics, Vertex, VertexProperty, GID,
    };
    use chrono::offset::TimeZone;
    use std::collections::HashMap;

    #[test]
    fn test_collections() {
        // List
        let value = json!({"@type": "g:List", "@value": [{"@type": "g:Int32", "@value": 1},
                                                      {"@type": "g:Int32", "@value": 2},
                                                      "3"]});

        let result = deserializer_v3(&value).expect("Failed to deserialize a List");

        assert_eq!(
            result,
            GValue::List(vec![
                GValue::Int32(1),
                GValue::Int32(2),
                GValue::String(String::from("3")),
            ])
        );

        // Set
        let value = json!({"@type": "g:Set", "@value": [{"@type": "g:Int32", "@value": 1},
                                                     {"@type": "g:Int32", "@value": 2},
                                                     {"@type": "g:Float", "@value": 2.0},
                                                     "3"]});

        let result = deserializer_v3(&value).expect("Failed to deserialize a Set");

        assert_eq!(
            result,
            GValue::List(vec![
                GValue::Int32(1),
                GValue::Int32(2),
                GValue::Float(2.0),
                GValue::String(String::from("3")),
            ])
        );

        // Map

        let value = json!({"@type": "g:Map",
                        "@value": ['a', {"@type": "g:Int32", "@value": 1}, 'b', "marko"]});

        let result = deserializer_v3(&value).expect("Failed to deserialize a Map");

        let mut map = HashMap::new();
        map.insert(String::from("a"), GValue::Int32(1));
        map.insert(String::from("b"), GValue::String(String::from("marko")));
        assert_eq!(result, GValue::Map(map));
    }

    #[test]
    fn test_number_input() {
        // I32
        let value = json!({
            "@type": "g:Int32",
            "@value": 31
        });

        let result = deserializer_v3(&value).expect("Failed to deserialize an Int32");
        assert_eq!(result, GValue::Int32(31));

        // I64
        let value = json!({
            "@type": "g:Int64",
            "@value": 31
        });

        let result = deserializer_v3(&value).expect("Failed to deserialize an Int64");
        assert_eq!(result, GValue::Int64(31));

        // F32
        let value = json!({
            "@type": "g:Float",
            "@value": 31.3
        });

        let result = deserializer_v3(&value).expect("Failed to deserialize Float");

        assert_eq!(result, GValue::Float(31.3));

        // F64
        let value = json!({
            "@type": "g:Double",
            "@value": 31.3
        });

        let result = deserializer_v3(&value).expect("Failed to deserialize Double");
        assert_eq!(result, GValue::Double(31.3));

        // Date
        let value = json!({
            "@type": "g:Date",
            "@value": 1551825863
        });

        let result = deserializer_v3(&value).expect("Failed to deserialize Double");
        assert_eq!(result, GValue::Date(chrono::Utc.timestamp(1551825863, 0)));

        // UUID
        let value = json!({
            "@type" : "g:UUID",
            "@value" : "41d2e28a-20a4-4ab0-b379-d810dede3786"
        });

        let result = deserializer_v3(&value).expect("Failed to deserialize Double");
        assert_eq!(
            result,
            GValue::Uuid(uuid::Uuid::parse_str("41d2e28a-20a4-4ab0-b379-d810dede3786").unwrap())
        );
    }

    #[test]
    fn test_properties() {
        let value = json!({"@type":"g:VertexProperty", "@value":{"id":{"@type":"g:Int32","@value":1},"label":"name","value":"marko"}});

        let result = deserializer_v3(&value).expect("Failed to deserialize a VertexProperty");

        assert_eq!(
            result,
            VertexProperty::new(
                GID::Int32(1),
                String::from("name"),
                GValue::String(String::from("marko"))
            )
            .into()
        );

        let value = json!({"@type":"g:Property","@value":{"key":"since","value":{"@type":"g:Int32","@value":2009}}});

        let result = deserializer_v3(&value).expect("Failed to deserialize a VertexProperty");

        assert_eq!(
            result,
            Property::new(String::from("since"), GValue::Int32(2009)).into()
        );
    }
    #[test]
    fn test_vertex() {
        let value = json!({"@type":"g:Vertex", "@value":{"id":{"@type":"g:Int32","@value":45}}});

        let result = deserializer_v3(&value).expect("Failed to deserialize a Vertex");

        assert_eq!(
            result,
            Vertex::new(GID::Int32(45), String::from("vertex"), HashMap::new()).into()
        );

        let value = r#"{"@type":"g:Vertex","@value":{"id":{"@type":"g:Int32","@value":1},"label":"person","properties":{"name":[{"@type":"g:VertexProperty","@value":{"id":{"@type":"g:Int64","@value":0},"value":"marko","label":"name"}}],"location":[{"@type":"g:VertexProperty","@value":{"id":{"@type":"g:Int64","@value":6},"value":"san diego","label":"location","properties":{"startTime":{"@type":"g:Int32","@value":1997},"endTime":{"@type":"g:Int32","@value":2001}}}},{"@type":"g:VertexProperty","@value":{"id":{"@type":"g:Int64","@value":7},"value":"santa cruz","label":"location","properties":{"startTime":{"@type":"g:Int32","@value":2001},"endTime":{"@type":"g:Int32","@value":2004}}}},{"@type":"g:VertexProperty","@value":{"id":{"@type":"g:Int64","@value":8},"value":"brussels","label":"location","properties":{"startTime":{"@type":"g:Int32","@value":2004},"endTime":{"@type":"g:Int32","@value":2005}}}},{"@type":"g:VertexProperty","@value":{"id":{"@type":"g:Int64","@value":9},"value":"santa fe","label":"location","properties":{"startTime":{"@type":"g:Int32","@value":2005}}}}]}}}"#;

        let val = serde_json::from_str(&value).expect("Failed to serialize");

        let result = deserializer_v3(&val).expect("Failed to deserialize a vertex");

        assert_eq!(
            result,
            vertex!({
                id => 1,
                label => "person",
                properties => {
                    "name" => [ { id => 0 as i64 , value => "marko"}],
                    "location" => [{ id => 6 as i64, value => "san diego"},{ id => 7  as i64 , value => "santa cruz"},{ id => 8  as i64, value => "brussels"},{ id => 9  as i64, value => "santa fe"}]
                }
            }).into()
        );
    }

    #[test]
    fn test_edge() {
        let value = json!({"@type":"g:Edge","@value":{"id":{"@type":"g:Int32","@value":13},"label":"develops","inVLabel":"software","outVLabel":"person","inV":{"@type":"g:Int32","@value":10},"outV":{"@type":"g:Int32","@value":1},"properties":{"since":{"@type":"g:Property","@value":{"key":"since","value":{"@type":"g:Int32","@value":2009}}}}}});

        let result = deserializer_v3(&value).expect("Failed to deserialize an Edge");

        assert_eq!(
            result,
            edge!({
                id => 13,
                label=> "develops",
                inV => {
                    id => 10,
                    label => "software"
                },
                outV => {
                    id => 1,
                    label => "person"
                },
                properties => {

                }
            })
            .into()
        );
    }

    #[test]
    fn test_path() {
        let value = json!({"@type":"g:Path","@value":{"labels":{"@type":"g:List","@value":[{"@type":"g:Set","@value":[]},{"@type":"g:Set","@value":[]},{"@type":"g:Set","@value":[]}]},"objects":{"@type":"g:List","@value":[{"@type":"g:Vertex","@value":{"id":{"@type":"g:Int32","@value":1},"label":"person"}},{"@type":"g:Vertex","@value":{"id":{"@type":"g:Int32","@value":10},"label":"software"}},{"@type":"g:Vertex","@value":{"id":{"@type":"g:Int32","@value":11},"label":"software"}}]}}});

        let result = deserializer_v3(&value).expect("Failed to deserialize a Path");

        let empty = GValue::List(vec![]);

        let path = Path::new(
            GValue::List(vec![empty.clone(), empty.clone(), empty.clone()]),
            vec![
                vertex!({ id => 1, label => "person", properties => {}}).into(),
                vertex!({ id => 10, label => "software", properties => {}}).into(),
                vertex!({ id => 11, label => "software", properties => {}}).into(),
            ],
        );
        assert_eq!(result, path.into());
    }

    #[test]
    fn test_traversal_metrics() {
        let value = serde_json::from_str(r#"{"@type":"g:TraversalMetrics","@value":{"@type":"g:Map","@value":["dur",{"@type":"g:Double","@value":0.004},"metrics",{"@type":"g:List","@value":[{"@type":"g:Metrics","@value":{"@type":"g:Map","@value":["dur",{"@type":"g:Double","@value":100.0},"counts",{"@type":"g:Map","@value":["traverserCount",{"@type":"g:Int64","@value":4},"elementCount",{"@type":"g:Int64","@value":4}]},"name","TinkerGraphStep(vertex,[~label.eq(person)])","annotations",{"@type":"g:Map","@value":["percentDur",{"@type":"g:Double","@value":25.0}]},"id","7.0.0()"]}},{"@type":"g:Metrics","@value":{"@type":"g:Map","@value":["dur",{"@type":"g:Double","@value":100.0},"counts",{"@type":"g:Map","@value":["traverserCount",{"@type":"g:Int64","@value":13},"elementCount",{"@type":"g:Int64","@value":13}]},"name","VertexStep(OUT,vertex)","annotations",{"@type":"g:Map","@value":["percentDur",{"@type":"g:Double","@value":25.0}]},"id","2.0.0()"]}},{"@type":"g:Metrics","@value":{"@type":"g:Map","@value":["dur",{"@type":"g:Double","@value":100.0},"counts",{"@type":"g:Map","@value":["traverserCount",{"@type":"g:Int64","@value":7},"elementCount",{"@type":"g:Int64","@value":7}]},"name","VertexStep(OUT,vertex)","annotations",{"@type":"g:Map","@value":["percentDur",{"@type":"g:Double","@value":25.0}]},"id","3.0.0()"]}},{"@type":"g:Metrics","@value":{"@type":"g:Map","@value":["dur",{"@type":"g:Double","@value":100.0},"counts",{"@type":"g:Map","@value":["traverserCount",{"@type":"g:Int64","@value":1},"elementCount",{"@type":"g:Int64","@value":1}]},"name","TreeStep","annotations",{"@type":"g:Map","@value":["percentDur",{"@type":"g:Double","@value":25.0}]},"id","4.0.0()"]}}]}]}}"#).expect("Error parsing json");

        let result = deserializer_v3(&value).expect("Failed to deserialize a TraversalMetrics");

        let traversal_metrics = TraversalMetrics::new(
            0.004,
            vec![
                Metric::new(
                    "7.0.0()",
                    "TinkerGraphStep(vertex,[~label.eq(person)])",
                    100.0,
                    4,
                    4,
                    25.0,
                ),
                Metric::new("2.0.0()", "VertexStep(OUT,vertex)", 100.0, 13, 13, 25.0),
                Metric::new("3.0.0()", "VertexStep(OUT,vertex)", 100.0, 7, 7, 25.0),
                Metric::new("4.0.0()", "TreeStep", 100.0, 1, 1, 25.0),
            ],
        );

        assert_eq!(result, traversal_metrics.into());
    }
}
