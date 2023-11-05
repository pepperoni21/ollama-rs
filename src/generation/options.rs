use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[derive(Default)]
pub struct GenerationOptions {
    pub(super) mirostat: Option<u8>,
    pub(super) mirostat_eta: Option<f32>,
    pub(super) mirostat_tau: Option<f32>,
    pub(super) num_ctx: Option<u32>,
    pub(super) num_gqa: Option<u32>,
    pub(super) num_gpu: Option<u32>,
    pub(super) num_thread: Option<u32>,
    pub(super) repeat_last_n: Option<i32>,
    pub(super) repeat_penalty: Option<f32>,
    pub(super) temperature: Option<f32>,
    pub(super) seed: Option<i32>,
    pub(super) stop: Option<String>,
    pub(super) tfs_z: Option<f32>,
    pub(super) num_predict: Option<i32>,
    pub(super) top_k: Option<u32>,
    pub(super) top_p: Option<f32>,
}



impl GenerationOptions {
    pub fn mirostat(mut self, mirostat: u8) -> Self {
        self.mirostat = Some(mirostat);
        self
    }

    pub fn mirostat_eta(mut self, mirostat_eta: f32) -> Self {
        self.mirostat_eta = Some(mirostat_eta);
        self
    }

    pub fn mirostat_tau(mut self, mirostat_tau: f32) -> Self {
        self.mirostat_tau = Some(mirostat_tau);
        self
    }

    pub fn num_ctx(mut self, num_ctx: u32) -> Self {
        self.num_ctx = Some(num_ctx);
        self
    }

    pub fn num_gqa(mut self, num_gqa: u32) -> Self {
        self.num_gqa = Some(num_gqa);
        self
    }

    pub fn num_gpu(mut self, num_gpu: u32) -> Self {
        self.num_gpu = Some(num_gpu);
        self
    }

    pub fn num_thread(mut self, num_thread: u32) -> Self {
        self.num_thread = Some(num_thread);
        self
    }

    pub fn repeat_last_n(mut self, repeat_last_n: i32) -> Self {
        self.repeat_last_n = Some(repeat_last_n);
        self
    }

    pub fn repeat_penalty(mut self, repeat_penalty: f32) -> Self {
        self.repeat_penalty = Some(repeat_penalty);
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn seed(mut self, seed: i32) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn stop(mut self, stop: String) -> Self {
        self.stop = Some(stop);
        self
    }

    pub fn tfs_z(mut self, tfs_z: f32) -> Self {
        self.tfs_z = Some(tfs_z);
        self
    }

    pub fn num_predict(mut self, num_predict: i32) -> Self {
        self.num_predict = Some(num_predict);
        self
    }

    pub fn top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }
}
