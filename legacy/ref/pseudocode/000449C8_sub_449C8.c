/* ea:   0x000449C8 */
/* name: sub_449C8 */
/* size: 172 */
_DWORD *__fastcall sub_449C8(int a1, _DWORD *a2, int a3, _DWORD *a4)
{
  _DWORD *v7; // r0
  _DWORD *v8; // r6
  int v9; // r1
  int v10; // r2
  int v11; // r3
  int v12; // r1
  int v13; // r2
  int v14; // r3
  int v15; // r1
  int v16; // r2
  int v17; // r3
  _DWORD *v18; // r0

  v7 = (_DWORD *)sub_44A74(a1, 1);
  v8 = v7;
  if ( v7 != (_DWORD *)-8 )
  {
    v9 = a4[1];
    v10 = a4[2];
    v11 = a4[3];
    v7[2] = *a4;
    v7[3] = v9;
    v7[4] = v10;
    v7[5] = v11;
    v12 = a4[5];
    v13 = a4[6];
    v14 = a4[7];
    v7[6] = a4[4];
    v7[7] = v12;
    v7[8] = v13;
    v7[9] = v14;
    v15 = a4[9];
    v16 = a4[10];
    v17 = a4[11];
    v7[10] = a4[8];
    v7[11] = v15;
    v7[12] = v16;
    v7[13] = v17;
    v7[14] = a4[12];
  }
  v18 = *(_DWORD **)(a3 + 4);
  *v8 = a3;
  v8[1] = v18;
  *v18 = v8;
  *(_DWORD *)(a3 + 4) = v8;
  *a2 = v8;
  return a2;
}

