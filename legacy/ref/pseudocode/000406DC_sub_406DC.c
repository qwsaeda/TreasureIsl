/* ea:   0x000406DC */
/* name: sub_406DC */
/* size: 96 */
char *__fastcall sub_406DC(int *a1, _DWORD *a2)
{
  _BYTE *v4; // r1
  char *result; // r0
  char v6; // [sp+8h] [bp-Ch] BYREF

  v4 = (_BYTE *)a1[1];
  if ( v4 == (_BYTE *)a1[2] )
    return sub_3854C(a1, v4, a2, (int)&v6, 1, 1);
  if ( v4 != nullptr )
    *(_DWORD *)v4 = *a2;
  result = (char *)a1[1];
  a1[1] = (int)(result + 4);
  return result;
}

