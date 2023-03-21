# pi_spine_rs
Spine 的 Rust 渲染

# wasm 版本使用
## 需要的单例资源

```rust
/// ### 可与其他渲染共用
/// * RenderDevice , RenderQueue
/// * BindBufferAllocator 
/// * VertexBufferAllocator
/// * Share<AssetMgr<TextureRes>>
/// * Share<AssetMgr<SamplerRes>>
/// * Share<AssetMgr<RenderRes<BindGroup>>>
/// 
/// ### 不可与其他渲染共用
/// * XHashMap<u32, Renderer>
/// * SingleSpineBindGroupLayout
/// * SingleSpinePipelinePool
```

# 其他渲染逻辑使用时

```rust
/// 通过 ID u32 获取 Renderer
/// 设置 renderpass , viewport, scissor
/// 调用 DrawList::render( &renderer.drawlist().list, &mut renderpass );
```
