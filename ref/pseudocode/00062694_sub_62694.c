/* ea:   0x00062694 */
/* name: sub_62694 */
/* size: 384 */
_DWORD *__fastcall sub_62694(int a1, BOOL a2, int a3, char a4)
{
  char v6; // r5
  int v7; // r6
  char v8; // r9
  bool v9; // cc
  unsigned int v10; // r0
  int v11; // r11
  int v12; // r10
  int v13; // r2
  int v14; // r4
  BOOL v15; // r0
  size_t v16; // r1
  _DWORD *result; // r0
  char *v18; // r2
  int v19; // r4
  char *v20; // r1
  unsigned int v22; // [sp+4h] [bp-28h]

  v6 = a4 & 3;
  v7 = (__int16)a3;
  v8 = 1;
  v9 = (__int16)a3 <= 8;
  v10 = (unsigned int)((__int16)a3 * a1) >> 3;
  v11 = v10 * a2;
  v22 = v10;
  v12 = 0;
  v13 = v10 * a2 + 40;
  if ( v9 )
  {
    v12 = 2 * (1 << v7);
    if ( (a4 & 2) != 2 )
      v12 = 3 * (1 << v7);
  }
  v14 = a4 & 1;
  v15 = v14 == 1;
  if ( v14 == 1 )
    v15 = a2;
  v16 = v13 + v12;
  if ( v14 == 1 )
    v16 += 4 * v15;
  result = malloc(v16);
  if ( result == nullptr )
    return nullptr;
  *(_BYTE *)result = v6;
  v18 = (char *)(result + 10);
  result[1] = a1;
  result[2] = a2;
  result[3] = a3;
  result[4] = v11;
  result[5] = v12;
  result[6] = v22;
  result[7] = result + 10;
  if ( v7 > 8 )
  {
    result[8] = 0;
  }
  else
  {
    if ( v12 == 0 )
      __debugbreak();
    result[8] = &v18[v11];
  }
  if ( v14 != 1 )
    v8 = 0;
  if ( v8 != 0 )
  {
    v19 = a2;
    v20 = &v18[v12 + v11];
    for ( result[9] = v20; v19 != 0; v18 += v22 )
    {
      --v19;
      *(_DWORD *)v20 = v18;
      v20 += 4;
    }
  }
  else
  {
    result[9] = 0;
  }
  return result;
}

