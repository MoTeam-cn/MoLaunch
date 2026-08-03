/**
 * 自定义布局解析器（对外统一入口）
 *
 * 支持 JSON 与 XML 两种格式，解析后统一转为 LayoutSchema；
 * 具体实现分置于 json-parser / xml-parser，schema 公共类型与校验常量见 schema-types。
 */
export * from './json-parser'
export * from './xml-parser'
export * from './schema-types'