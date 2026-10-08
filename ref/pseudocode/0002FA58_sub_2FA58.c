/* ea:   0x0002FA58 */
/* name: sub_2FA58 */
/* size: 152 */
int __fastcall sub_2FA58(_DWORD *a1, int *a2)
{
  int v4; // r6
  int v5; // r0
  _BYTE v7[8]; // [sp+0h] [bp-24h] BYREF
  _DWORD v8[7]; // [sp+8h] [bp-1Ch] BYREF

  v4 = sub_34AE4();
  if ( v4 != *a1 && *a2 >= *(_DWORD *)(v4 + 16) )
    return v4 + 20;
  v5 = *a2;
  v8[1] = 0;
  v8[0] = v5;
  v8[2] = 0;
  return *(_DWORD *)sub_37600(a1, v7, v4, v8) + 20;
}

