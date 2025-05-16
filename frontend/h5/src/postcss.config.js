module.exports = {
    plugins: {
      'postcss-pxtorem': {
        rootValue: 37.5, // Vant 基于 375px 设计稿
        propList: ['*'],
      },
      autoprefixer: {},
    },
}