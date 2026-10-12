/* ea:   0x0004B734 */
/* name: sub_4B734 */
/* size: 340 */
int __fastcall sub_4B734(int result, int a2, int a3)
{
  int v3; // r5
  __int16 v6; // r6
  int v7; // r0
  int v8; // r0

  v3 = result;
  if ( a3 != 0 )
  {
    v6 = 2;
    if ( dword_7FB6C == 2 )
    {
      v7 = 3;
    }
    else
    {
      v7 = 1;
      if ( *(_DWORD *)(v3 + 40) != *(_DWORD *)(v3 + 44) )
        v7 = 2;
    }
    *(_DWORD *)a2 = v7;
    *(_DWORD *)(a2 + 4) = *(_DWORD *)(v3 + 8);
    *(_DWORD *)(a2 + 20) = *(_DWORD *)(a3 + 8);
    *(_DWORD *)(a2 + 12) = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(v3 + 24) + 32) + 4 * *(_DWORD *)(v3 + 8) + 80);
    v8 = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(v3 + 24) + 32)
                   + 4 * (*(int (__fastcall **)(int))(*(_DWORD *)a3 + 136))(a3)
                   + 80);
    *(_DWORD *)(a2 + 28) = 0;
    *(_DWORD *)(a2 + 24) = v8;
    *(_DWORD *)(a2 + 8) = *(_DWORD *)(v3 + 36);
    *(_DWORD *)(a2 + 16) = *(_DWORD *)(a3 + 36);
    *(_WORD *)(a2 + 32) = (unsigned int)(sub_16C9C((unsigned __int16 *)(v3 + 136), 0) + 1) >> 1;
    *(_WORD *)(a2 + 34) = (unsigned int)sub_16C9C((unsigned __int16 *)(v3 + 136), 1) >> 1;
    if ( *(_DWORD *)(v3 + 176) != 8 )
      v6 = 1;
    *(_WORD *)(a2 + 36) = v6;
    *(_WORD *)(a2 + 38) = sub_16C9C((unsigned __int16 *)(a3 + 136), 0);
    result = sub_16C9C((unsigned __int16 *)(a3 + 136), 1);
    *(_WORD *)(a2 + 42) = 0;
    *(_WORD *)(a2 + 44) = 0;
    *(_WORD *)(a2 + 40) = result;
  }
  return result;
}

