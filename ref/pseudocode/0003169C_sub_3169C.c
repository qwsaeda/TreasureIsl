/* ea:   0x0003169C */
/* name: sub_3169C */
/* size: 340 */
int __fastcall sub_3169C(int a1)
{
  int result; // r0
  unsigned __int8 v3; // r9
  _DWORD *v4; // r3
  unsigned __int8 v5; // r7
  int v6; // r6
  int v7; // r5
  int v8; // r4
  int v9; // r4
  int v10; // r3
  int v11; // r0

  result = *(_DWORD *)(a1 + 20);
  if ( result == 0 )
    return result;
  v3 = 0;
  v4 = *(_DWORD **)(result + 28);
  v5 = 0;
  while ( v4 != *(_DWORD **)(result + 32) )
  {
    if ( *v4 == 7 )
    {
      ++v5;
LABEL_8:
      ++v4;
    }
    else
    {
      if ( *v4 != 9 )
        goto LABEL_8;
      ++v3;
      ++v4;
    }
  }
  v6 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v7 = rand();
  v8 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v9 = (unsigned __int16)((rand() % 10 + 1) * (unsigned int)v5 * v8 / 0x64);
  sub_16D78((unsigned __int16 *)(a1 + 136), v9 + (v7 % 5 + 1) * (unsigned int)v3 * v6 / 0x64);
  v10 = *(_DWORD *)(a1 + 20);
  v11 = 999;
  if ( v9 + *(unsigned __int16 *)(v10 + 22) <= 999 )
    v11 = v9 + *(unsigned __int16 *)(v10 + 22);
  result = v11 << 16;
  *(_WORD *)(v10 + 22) = HIWORD(result);
  return result;
}

