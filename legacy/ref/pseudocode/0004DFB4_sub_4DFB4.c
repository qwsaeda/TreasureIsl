/* ea:   0x0004DFB4 */
/* name: sub_4DFB4 */
/* size: 272 */
void __fastcall sub_4DFB4(void **a1, char *a2, int *a3)
{
  char *v4; // r7
  _BYTE *v6; // r5
  unsigned int v7; // r6
  char *v8; // r0
  char *v9; // r5
  char *v10; // r4
  char *v11; // r3
  unsigned int v12; // r2
  int *v13; // r4
  int v14; // r0
  int *i; // r1
  char v16; // [sp+4h] [bp-20h] BYREF

  v4 = a2;
  v6 = *a1;
  v7 = ((char *)a3 - a2) >> 2;
  if ( v7 <= ((_BYTE *)a1[2] - (_BYTE *)*a1) >> 2 )
  {
    v12 = ((_BYTE *)a1[1] - v6) >> 2;
    if ( v12 < v7 )
    {
      v13 = (int *)&a2[4 * v12];
      v14 = sub_4E0FC(a2, v13, *a1);
      for ( i = (int *)a1[1]; v13 != a3; ++i )
      {
        if ( i != nullptr )
          v14 = *v13;
        ++v13;
        if ( i != nullptr )
          *i = v14;
      }
      a1[1] = i;
    }
    else
    {
      a1[1] = (void *)sub_4E0C4(a2, a3, v6, &v16, 0);
    }
  }
  else
  {
    v8 = (char *)sub_132C8((int)(a1 + 2), v7);
    v9 = v8;
    if ( v4 != (char *)a3 )
    {
      v10 = (char *)(v8 - v4);
      do
      {
        sub_4E138(&v4[(_DWORD)v10], v4);
        v4 += 4;
      }
      while ( v4 != (char *)a3 );
    }
    sub_132B4(a1);
    *a1 = v9;
    v11 = &v9[4 * v7];
    a1[1] = v11;
    a1[2] = v11;
  }
}

