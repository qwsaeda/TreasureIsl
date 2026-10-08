/* ea:   0x0006AB64 */
/* name: sub_6AB64 */
/* size: 548 */
_DWORD *__fastcall sub_6AB64(int a1)
{
  int v2; // r7
  _DWORD *v3; // r0
  _DWORD *v4; // r6
  _BYTE *v6; // r3
  _WORD *v7; // r4
  int v8; // r5
  _WORD *v9; // r4
  _BYTE *v10; // r3
  char v11; // t1
  char v12; // t1
  char v13; // t1
  char v14; // t1
  char v15; // t1
  char v16; // t1
  char v17; // t1

  if ( a1 == 0 || *(_DWORD *)(a1 + 12) != 8 )
    __debugbreak();
  if ( a1 == 0 )
    __debugbreak();
  v2 = *(_DWORD *)(a1 + 8);
  if ( (*(_BYTE *)a1 & 2) == 2 )
    __debugbreak();
  v3 = sub_62694(*(_DWORD *)(a1 + 4), *(_DWORD *)(a1 + 8), 8, *(_BYTE *)a1 & 0xFC | 2);
  v4 = v3;
  if ( v3 == nullptr )
    return nullptr;
  v6 = *(_BYTE **)(a1 + 32);
  v7 = (_WORD *)v3[8];
  v8 = 32;
  do
  {
    *v7 = (8 * (v6[1] & 0xFC | (32 * (*v6 & 0xF8)))) | ((unsigned __int8)v6[2] >> 3);
    v9 = v7 + 1;
    v11 = v6[3];
    v10 = v6 + 3;
    *v9++ = (8 * (v10[1] & 0xFC | (32 * (v11 & 0xF8)))) | ((unsigned __int8)v10[2] >> 3);
    v12 = v10[3];
    v10 += 3;
    *v9++ = (8 * (v10[1] & 0xFC | (32 * (v12 & 0xF8)))) | ((unsigned __int8)v10[2] >> 3);
    v13 = v10[3];
    v10 += 3;
    *v9++ = (8 * (v10[1] & 0xFC | (32 * (v13 & 0xF8)))) | ((unsigned __int8)v10[2] >> 3);
    v14 = v10[3];
    v10 += 3;
    *v9++ = (8 * (v10[1] & 0xFC | (32 * (v14 & 0xF8)))) | ((unsigned __int8)v10[2] >> 3);
    v15 = v10[3];
    v10 += 3;
    *v9++ = (8 * (v10[1] & 0xFC | (32 * (v15 & 0xF8)))) | ((unsigned __int8)v10[2] >> 3);
    v16 = v10[3];
    v10 += 3;
    *v9++ = (8 * (v10[1] & 0xFC | (32 * (v16 & 0xF8)))) | ((unsigned __int8)v10[2] >> 3);
    v17 = v10[3];
    v10 += 3;
    --v8;
    *v9 = (8 * (v10[1] & 0xFC | (32 * (v17 & 0xF8)))) | ((unsigned __int8)v10[2] >> 3);
    v7 = v9 + 1;
    v6 = v10 + 3;
  }
  while ( v8 != 0 );
  memcpy((void *)v3[7], *(const void **)(a1 + 28), *(_DWORD *)(a1 + 24) * v2);
  return v4;
}

