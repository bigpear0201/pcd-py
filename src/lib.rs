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
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Cursor};

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
}

#[pyfunction]
fn read_pcd(path: String) -> PyResult<(MetaData, Py<PyDict>)> {
    // Optimization: Use Mmap for faster I/O and parallel decoding
    let reader =
        PcdReader::from_path_mmap(path).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let header = reader.header();

    let meta = MetaData {
        version: header.version.clone(),
        width: header.width,
        height: header.height,
        points: header.points,
        viewpoint: header.viewpoint.to_vec(),
    };

    let block = reader
        .read_all()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    Python::with_gil(|py| {
        let dict = PyDict::new(py);

        // Optimization: Consume columns to avoid extra clone()
        for (name, column) in block.columns {
            let py_array = match column {
                Column::F32(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::F64(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::U8(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::U16(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::U32(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::I8(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::I16(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::I32(v) => v.into_pyarray(py).to_owned().into_any(),
            };
            dict.set_item(name, py_array)?;
        }

        Ok((meta, dict.into()))
    })
}

#[pyfunction]
fn read_pcd_from_buffer(buffer: &Bound<'_, PyBytes>) -> PyResult<(MetaData, Py<PyDict>)> {
    let data = buffer.as_bytes();
    let mut cursor = Cursor::new(data);

    let header = parse_header(&mut cursor).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let layout =
        PcdLayout::from_header(&header).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let start_pos = cursor.position() as usize;

    let points = header.points;
    let mut block = PointBlock::new(
        &layout
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.type_))
            .collect(),
        points,
    );

    let data_slice = &data[start_pos..];

    match header.data {
        DataFormat::Binary => {
            let decoder = BinaryParallelDecoder::new(&layout, points);
            decoder
                .decode_par(data_slice, &mut block)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        DataFormat::BinaryCompressed => {
            // BinaryCompressed doesn't leverage parallel layout yet, use regular reader
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
    };

    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        for (name, column) in block.columns {
            let py_array = match column {
                Column::F32(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::F64(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::U8(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::U16(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::U32(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::I8(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::I16(v) => v.into_pyarray(py).to_owned().into_any(),
                Column::I32(v) => v.into_pyarray(py).to_owned().into_any(),
            };
            dict.set_item(name, py_array)?;
        }
        Ok((meta, dict.into()))
    })
}

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
        _ => return Err(PyTypeError::new_err("Unsupported format")),
    };

    let py = data.py();
    let mut fields = Vec::new();
    let mut points = 0;
    let mut columns = HashMap::new();

    for (key, value) in data.iter() {
        let name: String = key.extract()?;
        let array = value.downcast::<PyUntypedArray>().map_err(|_| {
            PyTypeError::new_err(format!("Value for field {} must be a numpy array", name))
        })?;

        // Ensure 1D array
        if array.ndim() != 1 {
            return Err(PyTypeError::new_err(format!(
                "Field {} must be a 1D array",
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
        let column = if dtype.is_equiv_to(&numpy::dtype::<f32>(py)) {
            let arr: &Bound<'_, PyArray1<f32>> = array.downcast().unwrap();
            fields.push((name.clone(), ValueType::F32));
            Column::F32(arr.to_vec()?)
        } else if dtype.is_equiv_to(&numpy::dtype::<f64>(py)) {
            let arr: &Bound<'_, PyArray1<f64>> = array.downcast().unwrap();
            fields.push((name.clone(), ValueType::F64));
            Column::F64(arr.to_vec()?)
        } else if dtype.is_equiv_to(&numpy::dtype::<u8>(py)) {
            let arr: &Bound<'_, PyArray1<u8>> = array.downcast().unwrap();
            fields.push((name.clone(), ValueType::U8));
            Column::U8(arr.to_vec()?)
        } else if dtype.is_equiv_to(&numpy::dtype::<u16>(py)) {
            let arr: &Bound<'_, PyArray1<u16>> = array.downcast().unwrap();
            fields.push((name.clone(), ValueType::U16));
            Column::U16(arr.to_vec()?)
        } else if dtype.is_equiv_to(&numpy::dtype::<u32>(py)) {
            let arr: &Bound<'_, PyArray1<u32>> = array.downcast().unwrap();
            fields.push((name.clone(), ValueType::U32));
            Column::U32(arr.to_vec()?)
        } else if dtype.is_equiv_to(&numpy::dtype::<i8>(py)) {
            let arr: &Bound<'_, PyArray1<i8>> = array.downcast().unwrap();
            fields.push((name.clone(), ValueType::I8));
            Column::I8(arr.to_vec()?)
        } else if dtype.is_equiv_to(&numpy::dtype::<i16>(py)) {
            let arr: &Bound<'_, PyArray1<i16>> = array.downcast().unwrap();
            fields.push((name.clone(), ValueType::I16));
            Column::I16(arr.to_vec()?)
        } else if dtype.is_equiv_to(&numpy::dtype::<i32>(py)) {
            let arr: &Bound<'_, PyArray1<i32>> = array.downcast().unwrap();
            fields.push((name.clone(), ValueType::I32));
            Column::I32(arr.to_vec()?)
        } else {
            return Err(PyTypeError::new_err(format!(
                "Unsupported numpy dtype for field {}",
                name
            )));
        };

        columns.insert(name, column);
    }

    let mut vp = [0.0; 7];
    if let Some(v) = viewpoint {
        if v.len() != 7 {
            return Err(PyRuntimeError::new_err("Viewpoint must have 7 elements"));
        }
        vp.copy_from_slice(&v);
    } else {
        vp = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
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

    let block = PointBlock {
        columns,
        len: points,
    };

    let file = File::create(path).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let writer = BufWriter::new(file);
    let mut pcd_writer = PcdWriter::new(writer);
    pcd_writer
        .write_pcd(&header, &block)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    Ok(())
}

#[pymodule]
fn pcd_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MetaData>()?;
    m.add_function(wrap_pyfunction!(read_pcd, m)?)?;
    m.add_function(wrap_pyfunction!(read_pcd_from_buffer, m)?)?;
    m.add_function(wrap_pyfunction!(write_pcd, m)?)?;
    Ok(())
}
