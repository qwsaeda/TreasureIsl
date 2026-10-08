/* ea:   0x00062814 */
/* name: sub_62814 */
/* size: 512 */
_DWORD *__fastcall sub_62814(int a1, int a2, int a3, char a4, char *a5, int a6)
{
  int v9; // r11
  unsigned int v10; // r1
  size_t v11; // r2
  int v12; // r6
  int v13; // r0
  int v14; // r10
  int v15; // r4
  int v16; // r5
  char v17; // r0
  _DWORD *v18; // r0
  _DWORD *v19; // r3
  int v21; // r2
  int v23; // r1
  int v24; // r4
  int v25; // r0
  _DWORD *v26; // r2
  int v28; // [sp+8h] [bp-2Ch]
  unsigned int v29; // [sp+Ch] [bp-28h]

  v9 = (__int16)a3;
  v10 = (__int16)a3 * a1;
  v11 = 40;
  v29 = v10 >> 3;
  v12 = a4 & 4;
  v28 = (v10 >> 3) * a2;
  v13 = v12 == 4;
  if ( v12 != 4 )
    v13 = (v10 >> 3) * a2;
  v14 = 0;
  if ( (a4 & 4) != 4 )
    v11 = v13 + 40;
  if ( v9 <= 8 )
  {
    v14 = 2 * (1 << v9);
    if ( (a4 & 2) != 2 )
      v14 = 3 * (1 << v9);
  }
  v15 = a4 & 8;
  v16 = a4 & 1;
  v17 = 1;
  if ( v15 != 8 )
    v11 += v14;
  if ( v16 != 1 )
    v17 = 0;
  if ( v17 != 0 )
    v11 += 4 * a2;
  v18 = malloc(v11);
  v19 = v18;
  if ( v18 == nullptr )
    return nullptr;
  *(_BYTE *)v18 = a4;
  v18[1] = a1;
  v18[2] = a2;
  v18[3] = a3;
  v21 = v28;
  v18[4] = v28;
  v18[5] = v14;
  v18[6] = v29;
  if ( v12 == 4 )
    v21 = 0;
  else
    a5 = (char *)(v18 + 10);
  v19[7] = a5;
  v23 = 0;
  if ( v9 > 8 )
  {
    v19[8] = 0;
  }
  else if ( v15 == 8 )
  {
    v19[8] = a6;
  }
  else
  {
    v19[8] = (char *)v19 + v21 + 40;
    v23 = v14;
  }
  if ( v16 == 1 )
  {
    v24 = a2;
    v25 = v19[7];
    v26 = (_DWORD *)((char *)v19 + v23 + v21 + 40);
    for ( v19[9] = v26; v24 != 0; v25 += v29 )
    {
      --v24;
      *v26++ = v25;
    }
  }
  else
  {
    v19[9] = 0;
  }
  return v19;
}

