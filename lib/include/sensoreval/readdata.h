#ifndef SENSOREVAL_READDATA_H
#define SENSOREVAL_READDATA_H

struct sensoreval_rd_ctx {
    uint8_t buf[sizeof(uint64_t) + sizeof(double)*9 + sizeof(uint64_t) + sizeof(double)*2 + sizeof(double)*4];
    size_t bufpos;
};

enum sensoreval_rd_ret {
    SENSOREVAL_RD_RET_OK = 0,
    SENSOREVAL_RD_RET_ERR,
    SENSOREVAL_RD_RET_WOULDBLOCK,
    SENSOREVAL_RD_RET_EOF,
};

static inline void sensoreval_rd_initctx(struct sensoreval_rd_ctx *ctx) {
    ctx->bufpos = 0;
}

enum sensoreval_rd_ret sensoreval_load_data_one(struct sensoreval_rd_ctx *ctx, int fd, struct sensoreval_data *psd);
int sensoreval_load_data(int fd, struct sensoreval_data **psdarr, size_t *psdarrsz);

#endif /* SENSOREVAL_READDATA_H */