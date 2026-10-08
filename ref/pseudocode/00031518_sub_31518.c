/* ea:   0x00031518 */
/* name: sub_31518 */
/* size: 388 */
int __fastcall sub_31518(int a1, _WORD *a2, _WORD *a3)
{
  int result; // r0
  unsigned __int8 v7; // r5
  _DWORD *v8; // r3
  unsigned __int8 v9; // r7
  int v10; // r4
  int v11; // r4
  __int16 v12; // r3
  int v13; // r4
  int v14; // r2
  int v15; // r3

  result = *(_DWORD *)(a1 + 20);
  if ( result == 0 )
    return result;
  v7 = 0;
  v8 = *(_DWORD **)(result + 28);
  v9 = 0;
  while ( v8 != *(_DWORD **)(result + 32) )
  {
    if ( *v8 == 7 )
    {
      ++v9;
LABEL_8:
      ++v8;
    }
    else
    {
      if ( *v8 != 9 )
        goto LABEL_8;
      ++v7;
      ++v8;
    }
  }
  v10 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  *a2 = (rand() % 5 + 1) * (unsigned int)v7 * v10 / 0x64;
  v11 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v12 = (rand() % 10 + 1) * (unsigned int)v9 * v11 / 0x64;
  *a3 = v12;
  sub_16D78((unsigned __int16 *)(a1 + 136), *a2 + v12);
  v13 = *(_DWORD *)(a1 + 20);
  v14 = *(unsigned __int16 *)(v13 + 22);
  v15 = 999;
  if ( v14 + (unsigned int)(unsigned __int16)*a3 <= 0x3E7 )
    v15 = v14 + (unsigned __int16)*a3;
  *(_WORD *)(v13 + 22) = v15;
  return v15 << 16;
}

