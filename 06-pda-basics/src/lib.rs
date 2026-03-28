use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

pub mod error;
pub mod instruction;
pub mod state;

use error::VoteError;
use instruction::VoteInstruction;
use state::{UserVote, VoteTopic};

// 声明程序入口点
entrypoint!(process_instruction);

/// 程序入口点
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 反序列化指令数据
    let instruction = VoteInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // 根据指令类型分发处理
    match instruction {
        VoteInstruction::CreateTopic { description, bump } => {
            msg!("指令: 创建投票主题");
            process_create_topic(program_id, accounts, description, bump)
        }
        VoteInstruction::Vote { option, bump } => {
            msg!("指令: 投票");
            process_vote(program_id, accounts, option, bump)
        }
    }
}

/// 处理创建投票主题指令
fn process_create_topic(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    description: String,
    bump: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let creator_account = next_account_info(accounts_iter)?;
    let topic_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // 检查创建者是否签名
    if !creator_account.is_signer {
        msg!("错误: 创建者必须签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 验证描述长度
    if description.len() > VoteTopic::MAX_DESCRIPTION_LEN {
        msg!("错误: 描述过长");
        return Err(ProgramError::InvalidArgument);
    }

    // 派生PDA地址
    let (expected_pda, expected_bump) = Pubkey::find_program_address(
        &[b"vote_topic", creator_account.key.as_ref()],
        program_id,
    );

    // 验证PDA地址
    if topic_account.key != &expected_pda {
        msg!("错误: PDA地址不匹配");
        return Err(VoteError::InvalidPDA.into());
    }

    // 验证bump seed
    if bump != expected_bump {
        msg!("错误: Bump seed不匹配");
        return Err(VoteError::InvalidPDA.into());
    }

    // 计算所需空间和租金
    let space = VoteTopic::space(description.len());
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    // 创建PDA账户
    invoke_signed(
        &system_instruction::create_account(
            creator_account.key,
            topic_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            creator_account.clone(),
            topic_account.clone(),
            system_program.clone(),
        ],
        &[&[b"vote_topic", creator_account.key.as_ref(), &[bump]]],
    )?;

    // 初始化投票主题数据
    let vote_topic = VoteTopic {
        is_initialized: true,
        creator: *creator_account.key,
        description,
        option_a_votes: 0,
        option_b_votes: 0,
        bump,
    };

    // 序列化并保存数据
    vote_topic.serialize(&mut &mut topic_account.data.borrow_mut()[..])?;

    msg!("投票主题创建成功");
    Ok(())
}

/// 处理投票指令
fn process_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    option: u8,
    bump: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let voter_account = next_account_info(accounts_iter)?;
    let topic_account = next_account_info(accounts_iter)?;
    let user_vote_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // 检查投票者是否签名
    if !voter_account.is_signer {
        msg!("错误: 投票者必须签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 验证投票选项
    if option > 1 {
        msg!("错误: 投票选项无效");
        return Err(VoteError::InvalidVoteOption.into());
    }

    // 检查主题账户所有权
    if topic_account.owner != program_id {
        msg!("错误: 主题账户不属于此程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 反序列化主题数据
    let mut vote_topic = {
        let data = topic_account.data.borrow();
        let mut data_slice = &data[..];
        VoteTopic::deserialize(&mut data_slice)?
    };

    // 检查主题是否已初始化
    if !vote_topic.is_initialized {
        msg!("错误: 主题未初始化");
        return Err(VoteError::UninitializedAccount.into());
    }

    // 派生用户投票记录PDA地址
    let (expected_pda, expected_bump) = Pubkey::find_program_address(
        &[
            b"user_vote",
            topic_account.key.as_ref(),
            voter_account.key.as_ref(),
        ],
        program_id,
    );

    // 验证PDA地址
    if user_vote_account.key != &expected_pda {
        msg!("错误: 用户投票记录PDA地址不匹配");
        return Err(VoteError::InvalidPDA.into());
    }

    // 验证bump seed
    if bump != expected_bump {
        msg!("错误: Bump seed不匹配");
        return Err(VoteError::InvalidPDA.into());
    }

    // 检查是否已经投过票
    // 如果账户已经被程序拥有且有数据，说明已经投过票
    if user_vote_account.owner == program_id && user_vote_account.data_len() > 0 {
        msg!("错误: 已经投过票");
        return Err(VoteError::AlreadyVoted.into());
    }

    // 只有当账户不存在时才创建
    if user_vote_account.data_len() == 0 {
        // 创建用户投票记录PDA账户
        let space = UserVote::space();
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(space);

        invoke_signed(
            &system_instruction::create_account(
                voter_account.key,
                user_vote_account.key,
                lamports,
                space as u64,
                program_id,
            ),
            &[
                voter_account.clone(),
                user_vote_account.clone(),
                system_program.clone(),
            ],
            &[&[
                b"user_vote",
                topic_account.key.as_ref(),
                voter_account.key.as_ref(),
                &[bump],
            ]],
        )?;
    }

    // 初始化用户投票记录
    let user_vote = UserVote {
        is_initialized: true,
        topic: *topic_account.key,
        voter: *voter_account.key,
        vote_option: option,
        bump,
    };

    // 序列化并保存用户投票记录
    user_vote.serialize(&mut &mut user_vote_account.data.borrow_mut()[..])?;

    // 更新投票计数
    if option == 0 {
        vote_topic.option_a_votes += 1;
        msg!("投票给选项A");
    } else {
        vote_topic.option_b_votes += 1;
        msg!("投票给选项B");
    }

    // 保存更新后的主题数据
    vote_topic.serialize(&mut &mut topic_account.data.borrow_mut()[..])?;

    msg!("投票成功");
    Ok(())
}
