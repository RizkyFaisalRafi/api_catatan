use serde::Serialize;


// Buat fungsi helper kecil ini di atas struct
// Fungsi ini akan memeriksa apakah tipe 'T' adalah tipe unit '()'
fn is_unit<T>(_: &T) -> bool {
    std::mem::size_of::<T>() == 0
}

// Struct generik ini akan menjadi wrapper JSON standar
// 'T' adalah tipe data generik (bisa Note, Vec<Note>, atau apapun)
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub message: String,
    // pub data: T,
    //    Ini memberi tahu serde: "Panggil fungsi 'is_unit'.
    //    Jika hasilnya true, JANGAN serialisasi field ini."
    #[serde(skip_serializing_if = "is_unit")]
    pub data: T,
}