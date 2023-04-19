fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let inv_len = if len == 0.0 { 0.0 } else { 1.0 / len }; // prevent divide by 0
    [v[0] * inv_len, v[1] * inv_len, v[2] * inv_len]
}

fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

// subdivide icosphere, convert each triangle into 4 new
fn subdivide_icosphere(
    vert: Vec<[f32; 3]>,
    tri: Vec<[usize; 3]>,
) -> (Vec<[f32; 3]>, Vec<[usize; 3]>) {
    let mut next_vert: Vec<[f32; 3]> = vec![];
    let mut next_tri: Vec<[usize; 3]> = vec![];
    for t in tri {
        let curr_ind = next_vert.len();
        next_vert.append(&mut vec![
            vert[t[0]],
            vert[t[1]],
            vert[t[2]],
            normalize(midpoint(vert[t[0]], vert[t[1]])),
            normalize(midpoint(vert[t[1]], vert[t[2]])),
            normalize(midpoint(vert[t[2]], vert[t[0]])),
        ]);
        next_tri.append(&mut vec![
            [curr_ind, 3 + curr_ind, 5 + curr_ind],
            [3 + curr_ind, 1 + curr_ind, 4 + curr_ind],
            [4 + curr_ind, 2 + curr_ind, 5 + curr_ind],
            [3 + curr_ind, 4 + curr_ind, 5 + curr_ind],
        ])
    }
    (next_vert, next_tri)
}

pub fn get_icosphere(iterations: usize) -> Vec<f32> {
    // precalculated values for normalized vertices
    const A: f32 = 0.5257311;
    const B: f32 = 0.8506508;
    // init starting geometry
    let mut vertices: Vec<[f32; 3]> = vec![
        [-A, B, 0.0],
        [A, B, 0.0],
        [-A, -B, 0.0],
        [A, -B, 0.0],
        [0.0, -A, B],
        [0.0, A, B],
        [0.0, -A, -B],
        [0.0, A, -B],
        [B, 0.0, -A],
        [B, 0.0, A],
        [-B, 0.0, -A],
        [-B, 0.0, A],
    ];
    let mut triangles: Vec<[usize; 3]> = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    for _ in 0..iterations {
        (vertices, triangles) = subdivide_icosphere(vertices, triangles);
    }
    // create buffer from triangle / vertex sets
    let mut buffer: Vec<f32> = vec![];
    for tri in triangles {
        for ind in tri {
            for val in vertices[ind] {
                buffer.push(val);
            }
        }
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        let v = [3.0, 0.0, 4.0];
        let normalized_v = normalize(v);
        let expected_v = [0.6, 0.0, 0.8];

        for i in 0..3 {
            assert!((normalized_v[i] - expected_v[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_midpoint() {
        let a = [1.0, 2.0, 3.0];
        let b = [3.0, 6.0, 9.0];
        let midpoint_v = midpoint(a, b);
        let expected_v = [2.0, 4.0, 6.0];

        for i in 0..3 {
            assert_eq!(midpoint_v[i], expected_v[i]);
        }
    }

    #[test]
    fn test_subdivide_icosphere() {
        let vertices: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let triangles: Vec<[usize; 3]> = vec![[0, 1, 2]];
        let (subdivided_vertices, subdivided_triangles) = subdivide_icosphere(vertices, triangles);

        assert_eq!(subdivided_vertices.len(), 6);
        assert_eq!(subdivided_triangles.len(), 4);
    }

    #[test]
    fn test_get_icosphere() {
        let iterations = 1;
        let icosphere = get_icosphere(iterations);

        assert_eq!(
            icosphere.len(),
            3 * 3 * 20 * (4usize.pow(iterations as u32))
        );
    }

    #[test]
    fn test_icosphere_vertex_normalized() {
        let iterations = 2;
        let icosphere = get_icosphere(iterations);

        for i in (0..icosphere.len()).step_by(3) {
            let x = icosphere[i];
            let y = icosphere[i + 1];
            let z = icosphere[i + 2];
            let len = (x * x + y * y + z * z).sqrt();
            assert!((len - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_icosphere_triangle_edge_length_variation() {
        fn edge_length(a: [f32; 3], b: [f32; 3]) -> f32 {
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        }

        let iterations = 2;
        let icosphere = get_icosphere(iterations);
        let mut edge_lengths: Vec<f32> = Vec::new();

        for i in (0..icosphere.len()).step_by(9) {
            let a = [icosphere[i], icosphere[i + 1], icosphere[i + 2]];
            let b = [icosphere[i + 3], icosphere[i + 4], icosphere[i + 5]];
            let c = [icosphere[i + 6], icosphere[i + 7], icosphere[i + 8]];

            let ab_length = edge_length(a, b);
            let bc_length = edge_length(b, c);
            let ca_length = edge_length(c, a);

            let avg_edge_length = (ab_length + bc_length + ca_length) / 3.0;
            edge_lengths.push(avg_edge_length);
        }

        let max_difference = edge_lengths
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
            - edge_lengths
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

        assert!(
        max_difference < 0.05, // large epsilon to account for normalization / f32 operations
        "The maximum difference between average edge lengths of triangles is {}, which exceeds the allowed threshold.",
        max_difference
    );
    }
}
