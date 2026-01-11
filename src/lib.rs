use numpy::{
    IntoPyArray, PyArray1, PyArrayDescrMethods, PyArrayMethods, PyUntypedArray,
    PyUntypedArrayMethods,
};
use rs_pcd::decoder::ascii::AsciiReader;
use rs_pcd::decoder::binary_par::BinaryParallelDecoder;
use rs_pcd::decoder::compressed::CompressedReader;

use pyo3::exceptions::{PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use rs_pcd::header::{DataFormat, PcdHeader, ValueType, parse_header};
use rs_pcd::io::{PcdReader, PcdWriter};
use rs_pcd::layout::PcdLayout;
use rs_pcd::storage::{Column, PointBlock};
use std::fs::File;
use std::io::{BufWriter, Cursor};

/// Python-accessible metadata from PCD header
#[pyclass]
#[derive(Clone)]
pub struct MetaData {
    #[pyo3(get)]
    pub version: String,
    #[pyo3(get)]
    pub width: u32,
    #[pyo3(get)]
    pub height: u32,
    #[pyo3(get)]
    pub points: usize,
    #[pyo3(get)]
    pub viewpoint: Vec<f64>,
    #[pyo3(get)]
    pub fields: Vec<String>,
}

/// Convert a Column reference to a PyObject (numpy array)
fn column_to_pyarray(py: Python<'_>, column: &Column) -> PyObject {
    match column {
        Column::F32(v) => v.clone().into_pyarray(py).into_any().unbind(),
        Column::F64(v) => v.clone().into_pyarray(py).into_any().unbind(),
        Column::U8(v) => v.clone().into_pyarray(py).into_any().unbind(),
        Column::U16(v) => v.clone().into_pyarray(py).into_any().unbind(),
        Column::U32(v) => v.clone().into_pyarray(py).into_any().unbind(),
        Column::I8(v) => v.clone().into_pyarray(py).into_any().unbind(),
        Column::I16(v) => v.clone().into_pyarray(py).into_any().unbind(),
        Column::I32(v) => v.clone().into_pyarray(py).into_any().unbind(),
    }
}

/// Read a PCD file from disk.
/// 
/// Uses memory-mapped I/O for maximum performance.
/// Returns (metadata, dict of numpy arrays).
#[pyfunction]
fn read_pcd(path: String) -> PyResult<(MetaData, Py<PyDict>)> {
    let reader =
        PcdReader::from_path_mmap(&path).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let header = reader.header();

    let meta = MetaData {
        version: header.version.clone(),
        width: header.width,
        height: header.height,
        points: header.points,
        viewpoint: header.viewpoint.to_vec(),
        fields: header.fields.clone(),
    };

    let block = reader
        .read_all()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    Python::with_gil(|py| {
        let dict = PyDict::new(py);

        // Use schema() and get_column_by_index() for iteration (v0.2.0 API)
        for (idx, name) in block.schema().iter().enumerate() {
            if let Some(column) = block.get_column_by_index(idx) {
                let py_array = column_to_pyarray(py, column);
                dict.set_item(name, py_array)?;
            }
        }

        Ok((meta, dict.into()))
    })
}

/// Read a PCD file from a bytes buffer.
/// 
/// Useful for reading from network streams or embedded resources.
/// Returns (metadata, dict of numpy arrays).
#[pyfunction]
fn read_pcd_from_buffer(buffer: &Bound<'_, PyBytes>) -> PyResult<(MetaData, Py<PyDict>)> {
    let data = buffer.as_bytes();
    let mut cursor = Cursor::new(data);

    let header = parse_header(&mut cursor).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let layout =
        PcdLayout::from_header(&header).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let start_pos = cursor.position() as usize;

    let points = header.points;
    let schema: Vec<(String, ValueType)> = layout
        .fields
        .iter()
        .map(|f| (f.name.clone(), f.type_))
        .collect();
    let mut block = PointBlock::new(&schema, points);

    let data_slice = &data[start_pos..];

    match header.data {
        DataFormat::Binary => {
            let decoder = BinaryParallelDecoder::new(&layout, points);
            decoder
                .decode_par(data_slice, &mut block)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        DataFormat::BinaryCompressed => {
            let mut cursor = Cursor::new(data_slice);
            let mut decoder = CompressedReader::new(&mut cursor, &layout, points);
            decoder
                .decode(&mut block)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        DataFormat::Ascii => {
            let mut cursor = Cursor::new(data_slice);
            let mut decoder = AsciiReader::new(&mut cursor, &layout, points);
            decoder
                .decode(&mut block)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
    }

    let meta = MetaData {
        version: header.version.clone(),
        width: header.width,
        height: header.height,
        points: header.points,
        viewpoint: header.viewpoint.to_vec(),
        fields: header.fields.clone(),
    };

    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        
        // Use schema() and get_column_by_index() for iteration (v0.2.0 API)
        for (idx, name) in block.schema().iter().enumerate() {
            if let Some(column) = block.get_column_by_index(idx) {
                let py_array = column_to_pyarray(py, column);
                dict.set_item(name, py_array)?;
            }
        }
        
        Ok((meta, dict.into()))
    })
}

/// Write a PCD file to disk.
/// 
/// Args:
///     path: Output file path
///     data: Dict of field_name -> numpy array
///     format: "ascii", "binary", or "binary_compressed"
///     viewpoint: Optional [tx, ty, tz, qw, qx, qy, qz] (default: identity)
#[pyfunction]
#[pyo3(signature = (path, data, format="binary", viewpoint=None))]
fn write_pcd(
    path: String,
    data: &Bound<'_, PyDict>,
    format: &str,
    viewpoint: Option<Vec<f64>>,
) -> PyResult<()> {
    let data_format = match format {
        "ascii" => DataFormat::Ascii,
        "binary" => DataFormat::Binary,
        "binary_compressed" => DataFormat::BinaryCompressed,
        _ => return Err(PyTypeError::new_err("Unsupported format. Use 'ascii', 'binary', or 'binary_compressed'")),
    };

    let py = data.py();
    let mut fields: Vec<(String, ValueType)> = Vec::new();
    let mut points = 0;
    let mut column_data: Vec<(String, Column)> = Vec::new();

    for (key, value) in data.iter() {
        let name: String = key.extract()?;
        let array = value.downcast::<PyUntypedArray>().map_err(|_| {
            PyTypeError::new_err(format!("Value for field '{}' must be a numpy array", name))
        })?;

        if array.ndim() != 1 {
            return Err(PyTypeError::new_err(format!(
                "Field '{}' must be a 1D array",
                name
            )));
        }

        let num_elements = array.shape()[0];
        if points == 0 {
            points = num_elements;
        } else if points != num_elements {
            return Err(PyRuntimeError::new_err(
                "All arrays must have the same length",
            ));
        }

        let dtype = array.dtype();
        let (vtype, column) = if dtype.is_equiv_to(&numpy::dtype::<f32>(py)) {
            let arr: &Bound<'_, PyArray1<f32>> = array.downcast().unwrap();
            (ValueType::F32, Column::F32(arr.to_vec()?))
        } else if dtype.is_equiv_to(&numpy::dtype::<f64>(py)) {
            let arr: &Bound<'_, PyArray1<f64>> = array.downcast().unwrap();
            (ValueType::F64, Column::F64(arr.to_vec()?))
        } else if dtype.is_equiv_to(&numpy::dtype::<u8>(py)) {
            let arr: &Bound<'_, PyArray1<u8>> = array.downcast().unwrap();
            (ValueType::U8, Column::U8(arr.to_vec()?))
        } else if dtype.is_equiv_to(&numpy::dtype::<u16>(py)) {
            let arr: &Bound<'_, PyArray1<u16>> = array.downcast().unwrap();
            (ValueType::U16, Column::U16(arr.to_vec()?))
        } else if dtype.is_equiv_to(&numpy::dtype::<u32>(py)) {
            let arr: &Bound<'_, PyArray1<u32>> = array.downcast().unwrap();
            (ValueType::U32, Column::U32(arr.to_vec()?))
        } else if dtype.is_equiv_to(&numpy::dtype::<i8>(py)) {
            let arr: &Bound<'_, PyArray1<i8>> = array.downcast().unwrap();
            (ValueType::I8, Column::I8(arr.to_vec()?))
        } else if dtype.is_equiv_to(&numpy::dtype::<i16>(py)) {
            let arr: &Bound<'_, PyArray1<i16>> = array.downcast().unwrap();
            (ValueType::I16, Column::I16(arr.to_vec()?))
        } else if dtype.is_equiv_to(&numpy::dtype::<i32>(py)) {
            let arr: &Bound<'_, PyArray1<i32>> = array.downcast().unwrap();
            (ValueType::I32, Column::I32(arr.to_vec()?))
        } else {
            return Err(PyTypeError::new_err(format!(
                "Unsupported numpy dtype for field '{}'. Supported: f32, f64, u8, u16, u32, i8, i16, i32",
                name
            )));
        };

        fields.push((name.clone(), vtype));
        column_data.push((name, column));
    }

    let mut vp = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
    if let Some(v) = viewpoint {
        if v.len() != 7 {
            return Err(PyRuntimeError::new_err("Viewpoint must have 7 elements: [tx, ty, tz, qw, qx, qy, qz]"));
        }
        vp.copy_from_slice(&v);
    }

    let header = PcdHeader {
        version: "0.7".to_string(),
        fields: fields.iter().map(|(n, _)| n.clone()).collect(),
        sizes: fields.iter().map(|(_, t)| t.size()).collect(),
        types: fields
            .iter()
            .map(|(_, t)| match t {
                ValueType::U8 | ValueType::U16 | ValueType::U32 => 'U',
                ValueType::I8 | ValueType::I16 | ValueType::I32 => 'I',
                ValueType::F32 | ValueType::F64 => 'F',
            })
            .collect(),
        counts: vec![1; fields.len()],
        width: points as u32,
        height: 1,
        viewpoint: vp,
        points,
        data: data_format,
    };

    // Create PointBlock using the new API
    let mut block = PointBlock::new(&fields, points);
    
    // Copy data into block columns
    for (name, src_column) in column_data {
        if let Some(dest_column) = block.get_column_mut(&name) {
            match (src_column, dest_column) {
                (Column::F32(src), Column::F32(dest)) => dest.copy_from_slice(&src),
                (Column::F64(src), Column::F64(dest)) => dest.copy_from_slice(&src),
                (Column::U8(src), Column::U8(dest)) => dest.copy_from_slice(&src),
                (Column::U16(src), Column::U16(dest)) => dest.copy_from_slice(&src),
                (Column::U32(src), Column::U32(dest)) => dest.copy_from_slice(&src),
                (Column::I8(src), Column::I8(dest)) => dest.copy_from_slice(&src),
                (Column::I16(src), Column::I16(dest)) => dest.copy_from_slice(&src),
                (Column::I32(src), Column::I32(dest)) => dest.copy_from_slice(&src),
                _ => return Err(PyRuntimeError::new_err("Type mismatch in column copy")),
            }
        }
    }

    let file = File::create(path).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let writer = BufWriter::new(file);
    let mut pcd_writer = PcdWriter::new(writer);
    pcd_writer
        .write_pcd(&header, &block)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    Ok(())
}

/// pcd-py: High-performance PCD I/O for Python
/// 
/// Functions:
///     read_pcd(path) -> (MetaData, dict)
///     read_pcd_from_buffer(bytes) -> (MetaData, dict)
///     write_pcd(path, data, format="binary", viewpoint=None)
#[pymodule]
fn pcd_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MetaData>()?;
    m.add_function(wrap_pyfunction!(read_pcd, m)?)?;
    m.add_function(wrap_pyfunction!(read_pcd_from_buffer, m)?)?;
    m.add_function(wrap_pyfunction!(write_pcd, m)?)?;
    Ok(())
}
