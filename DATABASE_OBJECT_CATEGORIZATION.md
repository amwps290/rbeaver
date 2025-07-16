# RBeaver 数据库对象分类功能

## 概述

我们成功为 RBeaver 实现了一个全面的数据库对象分类功能，该功能提供了类似 DBeaver 的专业数据库管理工具体验。这个功能支持 PostgreSQL 数据库的多种对象类型，并提供了层次化的树形结构显示。

## 🎯 实现的功能

### 1. 支持的对象类型
- **📋 Tables** - 用户定义的表，包含所有模式
- **👁 Views** - 常规视图和物化视图
- **⚙️ Functions** - 用户定义的函数和存储过程
- **⚡ Triggers** - 表触发器及其关联函数
- **🔢 Sequences** - 自增序列
- **🗂️ Indexes** - 所有表索引
- **📁 Schemas** - 数据库模式组织
- **🔧 System Catalog** - PostgreSQL 系统表和视图（预留）

### 2. 层次化树形结构
```
Database
├── Schema 1 (总对象数)
│   ├── 📋 Tables (15)
│   │   ├── users
│   │   ├── orders
│   │   └── products
│   ├── 👁 Views (3)
│   │   ├── user_orders_view
│   │   └── product_sales_summary (物化视图)
│   ├── ⚙️ Functions (8)
│   ├── ⚡ Triggers (5)
│   ├── 🔢 Sequences (4)
│   └── 🗂️ Indexes (25)
└── Schema 2 (总对象数)
    └── ...
```

### 3. 核心特性

#### 🔍 搜索和过滤
- 实时搜索功能，支持对象名称过滤
- 搜索时自动隐藏无匹配项的类别
- 大小写不敏感的搜索

#### 📊 对象计数显示
- 每个类别显示对象数量（如 "Tables (15)"）
- 模式级别显示总对象数
- 空类别自动隐藏

#### 🖱️ 右键上下文菜单
每种对象类型都有专门的上下文菜单：
- **复制名称** - 复制对象名称到剪贴板
- **查看定义** - 显示对象定义（预留）
- **属性** - 显示对象属性（预留）
- **刷新** - 刷新对象（预留）
- 特定操作（如函数执行、序列下一个值等）

#### ⚡ 懒加载性能优化
- 按需加载对象数据
- 展开类别时才触发数据加载
- 支持大型数据库的高效浏览

#### 🎨 一致的 UI 设计
- 遵循 RBeaver 的浅色主题
- 每种对象类型都有专门的图标
- 悬停提示显示详细信息
- DBeaver 风格的导航模式

## 🏗️ 技术实现

### 1. 数据模型设计
创建了完整的数据结构来表示各种 PostgreSQL 对象：

<augment_code_snippet path="src/database/traits.rs" mode="EXCERPT">
````rust
/// View information
#[derive(Debug, Clone)]
pub struct View {
    pub name: String,
    pub schema: String,
    pub view_type: ViewType,
    pub definition: Option<String>,
    pub comment: Option<String>,
    pub owner: Option<String>,
    pub is_updatable: bool,
}

/// Function information
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub schema: String,
    pub function_type: FunctionType,
    pub return_type: String,
    pub arguments: Vec<FunctionArgument>,
    pub language: String,
    pub definition: Option<String>,
    pub comment: Option<String>,
    pub owner: Option<String>,
}
````
</augment_code_snippet>

### 2. PostgreSQL 元数据查询
实现了优化的 SQL 查询来从 PostgreSQL 系统目录获取元数据：

<augment_code_snippet path="src/database/postgresql_queries.rs" mode="EXCERPT">
````rust
/// Query to get views in a specific schema
pub const GET_VIEWS_QUERY: &str = r#"
SELECT 
    v.table_name as name,
    v.table_schema as schema,
    CASE 
        WHEN c.relkind = 'm' THEN 'MATERIALIZED VIEW'
        ELSE 'VIEW'
    END as view_type,
    v.view_definition as definition,
    COALESCE(obj_description(c.oid), '') as comment,
    pg_get_userbyid(c.relowner) as owner,
    v.is_updatable = 'YES' as is_updatable
FROM information_schema.views v
LEFT JOIN pg_class c ON c.relname = v.table_name
LEFT JOIN pg_namespace n ON n.oid = c.relnamespace AND n.nspname = v.table_schema
WHERE v.table_schema = $1
"#;
````
</augment_code_snippet>

### 3. 树形 UI 组件
扩展了 DatabaseTree 组件以支持新的对象类别：

<augment_code_snippet path="src/ui/database_tree.rs" mode="EXCERPT">
````rust
fn render_object_category(
    &mut self,
    ui: &mut Ui,
    connection_id: &str,
    schema_name: &str,
    category: ObjectCategory,
    counts: &ObjectCounts,
) {
    let (icon, label, count) = match category {
        ObjectCategory::Tables => ("📋", "Tables", counts.tables),
        ObjectCategory::Views => ("👁", "Views", counts.views + counts.materialized_views),
        ObjectCategory::Functions => ("⚙️", "Functions", counts.functions + counts.procedures),
        // ...
    };
    
    let category_label = format!("{} {} ({})", icon, label, count);
    // ...
}
````
</augment_code_snippet>

### 4. 主应用集成
将新功能集成到主应用的树展开处理和查询生成机制中：

<augment_code_snippet path="src/app.rs" mode="EXCERPT">
````rust
// Get schemas that need object loading
let objects_to_load = self.database_tree.get_schemas_needing_objects();
for (connection_id, schema_name, category) in objects_to_load {
    if let Some(connection) = self.connections.get(&connection_id) {
        match category {
            ObjectCategory::Views => {
                match self.runtime.block_on(connection.get_views(&schema_name)) {
                    Ok(views) => {
                        self.database_tree.set_views(&connection_id, schema_name.clone(), views);
                        log::info!("Loaded views for schema: {} in connection: {}", schema_name, connection_id);
                    }
                    // ...
                }
            }
            // ...
        }
    }
}
````
</augment_code_snippet>

## 🧪 测试

创建了专门的测试应用程序来演示功能：

<augment_code_snippet path="src/bin/test_object_categorization.rs" mode="EXCERPT">
````rust
/// Test binary for database object categorization feature
/// 
/// This test demonstrates the new object categorization functionality
/// including hierarchical tree structure, object counts, and search capabilities.

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Object Categorization Test",
        options,
        Box::new(|_cc| Ok(Box::new(ObjectCategorizationTestApp::new()))),
    )
}
````
</augment_code_snippet>

## 🚀 使用方法

1. **启动测试应用**：
   ```bash
   cargo run --bin test_object_categorization
   ```

2. **浏览对象**：
   - 展开模式查看对象类别
   - 点击类别查看具体对象
   - 右键点击对象查看上下文菜单

3. **搜索功能**：
   - 点击搜索图标 (🔍) 启用搜索
   - 输入对象名称进行过滤
   - 点击 "Clear" 清除搜索

## 📈 性能优化

- **懒加载**：只在需要时加载对象数据
- **缓存**：已加载的数据会被缓存
- **批量查询**：优化的 SQL 查询减少数据库往返
- **增量更新**：支持增量数据更新

## 🔮 未来扩展

- 支持更多数据库类型（MySQL、SQLite 等）
- 实现对象定义查看功能
- 添加对象属性编辑功能
- 支持对象依赖关系图
- 实现对象搜索历史记录

## 📝 总结

这个数据库对象分类功能为 RBeaver 提供了专业级的数据库浏览体验，支持：

✅ **层次化组织** - Database → Schemas → Object Types → Objects  
✅ **对象计数显示** - 如 "Tables (15)", "Views (3)"  
✅ **多种对象类型** - Tables, Views, Functions, Triggers, Sequences, Indexes  
✅ **搜索和过滤** - 实时搜索功能  
✅ **上下文菜单** - 对象特定的操作  
✅ **浅色主题** - 一致的样式设计  
✅ **懒加载** - 性能优化支持  

该功能已完全集成到 RBeaver 中，可以立即使用并为用户提供类似 DBeaver 的专业数据库管理体验。
