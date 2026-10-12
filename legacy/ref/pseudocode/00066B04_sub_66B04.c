/* ea:   0x00066B04 */
/* name: sub_66B04 */
/* size: 152 */
int __fastcall sub_66B04(int a1, int *a2, int a3, int a4)
{
  int v6; // r0
  int result; // r0
  int v8; // r4
  _DWORD v9[17]; // [sp+0h] [bp-44h] BYREF

  v6 = *a2;
  v9[0] = a3;
  v9[8] = 0;
  v9[9] = 0;
  v9[4] = v6;
  v9[1] = a4;
  v9[3] = a1;
  result = sub_66DA8(v9, a114, 56);
  if ( result == 0 )
  {
    v8 = sub_66DBC(v9, 4);
    if ( v8 == 1 )
    {
      *a2 = v9[5];
      return sub_66C00(v9);
    }
    else
    {
      sub_66C00(v9);
      if ( v8 == 0 )
        return -5;
      return v8;
    }
  }
  return result;
}

