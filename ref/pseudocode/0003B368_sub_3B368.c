/* ea:   0x0003B368 */
/* name: sub_3B368 */
/* size: 180 */
int __fastcall sub_3B368(int a1, _BYTE *a2, _BYTE *a3, char *a4, int a5)
{
  int v8; // r4
  int v9; // r1
  char *v11; // r4
  int v12; // r1
  _BYTE v13[28]; // [sp+4h] [bp-1Ch] BYREF

  v8 = (a5 - (int)a4) >> 1;
  v9 = (a3 - a2) >> 1;
  if ( v9 < v8 )
  {
    v11 = &a4[2 * v9];
    v12 = (2 * v9) >> 1;
    if ( v12 != 0 )
      memcpy(a2, a4, 2 * v12);
    sub_3B66C(a1, a3, v11, a5, v13);
    return a1;
  }
  else
  {
    if ( v8 != 0 )
      memcpy(a2, a4, 2 * v8);
    sub_13248(a1, &a2[2 * v8], a3);
    return a1;
  }
}

