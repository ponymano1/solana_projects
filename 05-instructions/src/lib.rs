use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod error;
pub mod instruction;
pub mod state;

use error::TodoError;
use instruction::TodoInstruction;
use state::{TodoItem, TodoList, MAX_DESCRIPTION_LEN, MAX_TITLE_LEN, MAX_TODOS};

// 声明程序入口点
entrypoint!(process_instruction);

/// 程序入口点
///
/// 所有对程序的调用都会通过这个函数
///
/// # 参数
/// * `program_id` - 当前程序的公钥
/// * `accounts` - 指令需要的账户列表
/// * `instruction_data` - 序列化的指令数据
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 反序列化指令数据
    let instruction = TodoInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // 根据指令类型分发处理
    match instruction {
        TodoInstruction::Initialize => {
            msg!("指令: 初始化Todo列表");
            process_initialize(program_id, accounts)
        }
        TodoInstruction::CreateTodo { title, description } => {
            msg!("指令: 创建Todo");
            process_create_todo(program_id, accounts, title, description)
        }
        TodoInstruction::UpdateTodo { id, completed } => {
            msg!("指令: 更新Todo");
            process_update_todo(program_id, accounts, id, completed)
        }
        TodoInstruction::DeleteTodo { id } => {
            msg!("指令: 删除Todo");
            process_delete_todo(program_id, accounts, id)
        }
    }
}

/// 处理初始化指令
fn process_initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner_account = next_account_info(accounts_iter)?;
    let todo_list_account = next_account_info(accounts_iter)?;

    // 检查所有者是否签名
    if !owner_account.is_signer {
        msg!("错误: 所有者必须签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 检查账户所有权
    if todo_list_account.owner != program_id {
        msg!("错误: 账户不属于此程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 反序列化账户数据
    let mut todo_list = {
        let data = todo_list_account.data.borrow();
        let mut data_slice = &data[..];
        TodoList::deserialize(&mut data_slice)
            .unwrap_or(TodoList {
                is_initialized: false,
                owner: Pubkey::default(),
                todos: Vec::new(),
                next_id: 0,
            })
    };

    // 检查是否已初始化
    if todo_list.is_initialized {
        msg!("错误: 账户已经初始化");
        return Err(TodoError::AlreadyInitialized.into());
    }

    // 初始化Todo列表
    todo_list.is_initialized = true;
    todo_list.owner = *owner_account.key;
    todo_list.todos = Vec::new();
    todo_list.next_id = 0;

    // 序列化并保存数据
    todo_list.serialize(&mut &mut todo_list_account.data.borrow_mut()[..])?;

    msg!("Todo列表初始化成功");
    Ok(())
}

/// 处理创建Todo指令
fn process_create_todo(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    title: String,
    description: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner_account = next_account_info(accounts_iter)?;
    let todo_list_account = next_account_info(accounts_iter)?;

    // 检查所有者是否签名
    if !owner_account.is_signer {
        msg!("错误: 所有者必须签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 检查账户所有权
    if todo_list_account.owner != program_id {
        msg!("错误: 账户不属于此程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 反序列化账户数据
    let mut todo_list = {
        let data = todo_list_account.data.borrow();
        let mut data_slice = &data[..];
        TodoList::deserialize(&mut data_slice)?
    };

    // 检查是否已初始化
    if !todo_list.is_initialized {
        msg!("错误: 账户未初始化");
        return Err(TodoError::UninitializedAccount.into());
    }

    // 检查权限
    if todo_list.owner != *owner_account.key {
        msg!("错误: 权限不足");
        return Err(TodoError::Unauthorized.into());
    }

    // 验证参数
    if title.len() > MAX_TITLE_LEN {
        msg!("错误: 标题过长");
        return Err(TodoError::TitleTooLong.into());
    }

    if description.len() > MAX_DESCRIPTION_LEN {
        msg!("错误: 描述过长");
        return Err(TodoError::DescriptionTooLong.into());
    }

    // 检查Todo列表是否已满
    if todo_list.todos.len() >= MAX_TODOS {
        msg!("错误: Todo列表已满");
        return Err(TodoError::TodoListFull.into());
    }

    // 创建新的Todo项
    let new_todo = TodoItem {
        id: todo_list.next_id,
        title,
        description,
        completed: false,
    };

    todo_list.todos.push(new_todo);
    todo_list.next_id += 1;

    // 序列化并保存数据
    todo_list.serialize(&mut &mut todo_list_account.data.borrow_mut()[..])?;

    msg!("Todo创建成功, ID: {}", todo_list.next_id - 1);
    Ok(())
}

/// 处理更新Todo指令
fn process_update_todo(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: u32,
    completed: bool,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner_account = next_account_info(accounts_iter)?;
    let todo_list_account = next_account_info(accounts_iter)?;

    // 检查所有者是否签名
    if !owner_account.is_signer {
        msg!("错误: 所有者必须签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 检查账户所有权
    if todo_list_account.owner != program_id {
        msg!("错误: 账户不属于此程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 反序列化账户数据
    let mut todo_list = {
        let data = todo_list_account.data.borrow();
        let mut data_slice = &data[..];
        TodoList::deserialize(&mut data_slice)?
    };

    // 检查是否已初始化
    if !todo_list.is_initialized {
        msg!("错误: 账户未初始化");
        return Err(TodoError::UninitializedAccount.into());
    }

    // 检查权限
    if todo_list.owner != *owner_account.key {
        msg!("错误: 权限不足");
        return Err(TodoError::Unauthorized.into());
    }

    // 查找并更新Todo
    let todo = todo_list
        .todos
        .iter_mut()
        .find(|t| t.id == id)
        .ok_or(TodoError::TodoNotFound)?;

    todo.completed = completed;

    // 序列化并保存数据
    todo_list.serialize(&mut &mut todo_list_account.data.borrow_mut()[..])?;

    msg!("Todo更新成功, ID: {}, 完成状态: {}", id, completed);
    Ok(())
}

/// 处理删除Todo指令
fn process_delete_todo(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: u32,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner_account = next_account_info(accounts_iter)?;
    let todo_list_account = next_account_info(accounts_iter)?;

    // 检查所有者是否签名
    if !owner_account.is_signer {
        msg!("错误: 所有者必须签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 检查账户所有权
    if todo_list_account.owner != program_id {
        msg!("错误: 账户不属于此程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 反序列化账户数据
    let mut todo_list = {
        let data = todo_list_account.data.borrow();
        let mut data_slice = &data[..];
        TodoList::deserialize(&mut data_slice)?
    };

    // 检查是否已初始化
    if !todo_list.is_initialized {
        msg!("错误: 账户未初始化");
        return Err(TodoError::UninitializedAccount.into());
    }

    // 检查权限
    if todo_list.owner != *owner_account.key {
        msg!("错误: 权限不足");
        return Err(TodoError::Unauthorized.into());
    }

    // 查找并删除Todo
    let index = todo_list
        .todos
        .iter()
        .position(|t| t.id == id)
        .ok_or(TodoError::TodoNotFound)?;

    todo_list.todos.remove(index);

    // 序列化并保存数据
    todo_list.serialize(&mut &mut todo_list_account.data.borrow_mut()[..])?;

    msg!("Todo删除成功, ID: {}", id);
    Ok(())
}
