/* ea:   0x000588E4 */
/* name: sub_588E4 */
/* size: 296 */
int __fastcall sub_588E4(int a1, _WORD *a2, unsigned __int8 a3, __int16 *a4)
{
  __int16 v6; // r6
  int v7; // r2
  int v8; // r0
  int v9; // r4
  int v10; // r1
  int v11; // kr00_4
  int v12; // r0
  int v13; // r3
  int v14; // r2
  int v15; // r5
  bool v16; // cc
  int v17; // kr04_4
  int v18; // r2
  _DWORD v20[22]; // [sp+0h] [bp-58h]

  v20[4] = 307;
  v20[12] = 307;
  v6 = a2[2];
  v20[8] = 768;
  v20[6] = 512;
  v20[10] = 512;
  v20[7] = 614;
  v20[9] = 614;
  v7 = *a4;
  v20[5] = 409;
  v20[11] = 409;
  v8 = a4[1];
  v9 = a3;
  v10 = (__int16)a2[3];
  v20[0] = 230;
  v20[1] = 230;
  v20[2] = 230;
  v20[3] = 230;
  v20[13] = 230;
  v20[14] = 230;
  v11 = v7 * v6 + v10 * v8;
  v12 = (unsigned __int16)a2[1];
  v13 = v11 / 256;
  v20[15] = 230;
  a2[3] = v6;
  v14 = (unsigned __int16)v12;
  if ( (a3 & 8) != 0 )
  {
    v12 = a3 - 16;
    v15 = v13 + v12 * v14;
  }
  else
  {
    v15 = v13 + (unsigned __int16)v12 * a3;
  }
  v16 = v15 <= -32768;
  if ( v15 < -32768 )
  {
    v15 = -32768;
  }
  else
  {
    v12 = 0x7FFF;
    v16 = v15 <= 0x7FFF;
  }
  if ( !v16 )
    v15 = v12;
  v17 = v20[v9] * v14;
  v18 = v17 / 256;
  if ( v17 / 256 < 16 )
    LOWORD(v18) = 16;
  a2[1] = v18;
  a2[2] = v15;
  return v15;
}

