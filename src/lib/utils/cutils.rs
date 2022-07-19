
pub mod cutils {

    // old name was chunkQStringList
    pub fn chunk_to_vectors(values: Vec<String>, chunk_size: u64) -> Vec<Vec<String>>
    {
        let mut out: Vec<Vec<String>> = vec![];
        let the_len = values.len() as u64;
        let mut chunks_count: u64 = (the_len / chunk_size) as u64;
        if (the_len % chunk_size) != 0 {
            chunks_count += 1;
        }


        for i in 0..chunks_count {
            let mut end_index = 0;
            if (((i + 1) * chunk_size) < the_len) {
                end_index = ((i + 1) * chunk_size);
            } else {
                end_index = the_len;
            }
            let mut a_chunk:Vec<String> = values[(i * chunk_size) as usize..end_index as usize].to_vec();
            out.push(a_chunk);

        }
        return out;
    }
}