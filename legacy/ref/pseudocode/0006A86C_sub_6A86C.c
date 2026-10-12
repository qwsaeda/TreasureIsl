/* ea:   0x0006A86C */
/* name: sub_6A86C */
/* size: 760 */
_DWORD *__fastcall sub_6A86C(int a1)
{
  int v2; // r8
  _DWORD *v3; // r0
  _DWORD *v4; // r7
  unsigned __int16 *v6; // r4
  _BYTE *v7; // r5
  int v8; // r6
  unsigned __int16 *v9; // r4
  unsigned __int16 v10; // t1
  unsigned __int16 v11; // t1
  _BYTE *v12; // r5
  unsigned __int16 v13; // t1
  unsigned __int16 v14; // t1
  unsigned __int16 v15; // t1
  unsigned __int16 v16; // t1
  unsigned __int16 v17; // t1
  unsigned __int16 v18; // t1

  if ( a1 == 0 || *(_DWORD *)(a1 + 12) != 8 )
    __debugbreak();
  if ( a1 == 0 )
    __debugbreak();
  v2 = *(_DWORD *)(a1 + 8);
  if ( (*(_BYTE *)a1 & 2) != 2 )
    __debugbreak();
  v3 = sub_62694(*(_DWORD *)(a1 + 4), *(_DWORD *)(a1 + 8), 8, *(_BYTE *)a1 & 0xFC);
  v4 = v3;
  if ( v3 == nullptr )
    return nullptr;
  v6 = *(unsigned __int16 **)(a1 + 32);
  v7 = (_BYTE *)v3[8];
  v8 = 32;
  do
  {
    v10 = *v6;
    v9 = v6 + 1;
    *v7 = HIBYTE(v10) & 0xF8;
    v7[1] = (v10 >> 3) & 0xFC;
    v7[2] = 8 * v10;
    v11 = *v9++;
    v7[3] = HIBYTE(v11) & 0xF8;
    v12 = v7 + 3;
    v12[1] = (v11 >> 3) & 0xFC;
    v12[2] = 8 * v11;
    v13 = *v9++;
    v12[3] = HIBYTE(v13) & 0xF8;
    v12 += 3;
    v12[1] = (v13 >> 3) & 0xFC;
    v12[2] = 8 * v13;
    v14 = *v9++;
    v12[3] = HIBYTE(v14) & 0xF8;
    v12 += 3;
    v12[1] = (v14 >> 3) & 0xFC;
    v12[2] = 8 * v14;
    v15 = *v9++;
    v12[3] = HIBYTE(v15) & 0xF8;
    v12 += 3;
    v12[1] = (v15 >> 3) & 0xFC;
    v12[2] = 8 * v15;
    v16 = *v9++;
    --v8;
    v12[3] = HIBYTE(v16) & 0xF8;
    v12 += 3;
    v12[1] = (v16 >> 3) & 0xFC;
    v12[2] = 8 * v16;
    v17 = *v9++;
    v12[3] = HIBYTE(v17) & 0xF8;
    v12 += 3;
    v12[1] = (v17 >> 3) & 0xFC;
    v12[2] = 8 * v17;
    v18 = *v9;
    v6 = v9 + 1;
    v12[3] = HIBYTE(v18) & 0xF8;
    v12 += 3;
    v12[1] = (v18 >> 3) & 0xFC;
    v12[2] = 8 * v18;
    v7 = v12 + 3;
  }
  while ( v8 != 0 );
  memcpy((void *)v3[7], *(const void **)(a1 + 28), *(_DWORD *)(a1 + 24) * v2);
  return v4;
}

