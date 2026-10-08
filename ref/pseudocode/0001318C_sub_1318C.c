/* ea:   0x0001318C */
/* name: sub_1318C */
/* size: 188 */
void **__fastcall sub_1318C(void **a1, char *a2, int a3)
{
  void *v6; // r3
  unsigned int v7; // r4
  unsigned int v8; // r0
  char v10; // [sp+0h] [bp-18h] BYREF

  v6 = *a1;
  v7 = (a3 - (int)a2) >> 1;
  v8 = ((_BYTE *)a1[1] - (_BYTE *)*a1) >> 1;
  if ( v7 > v8 )
  {
    if ( v8 != 0 )
      memcpy(v6, a2, 2 * v8);
    sub_136F8(a1, &a2[2 * (((_BYTE *)a1[1] - (_BYTE *)*a1) >> 1)], a3, &v10);
    return a1;
  }
  else
  {
    if ( v7 != 0 )
      memcpy(v6, a2, 2 * v7);
    sub_13248((int)a1, (char *)*a1 + 2 * v7, a1[1]);
    return a1;
  }
}

