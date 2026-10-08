/* ea:   0x0002F944 */
/* name: sub_2F944 */
/* size: 148 */
int __fastcall sub_2F944(_DWORD *a1, int *a2)
{
  int v4; // r6
  int v5; // r0
  _BYTE v7[8]; // [sp+0h] [bp-20h] BYREF
  _DWORD v8[6]; // [sp+8h] [bp-18h] BYREF

  v4 = sub_34A1C();
  if ( v4 != *a1 && *a2 >= *(_DWORD *)(v4 + 16) )
    return v4 + 20;
  v5 = *a2;
  v8[1] = 0;
  v8[0] = v5;
  return *(_DWORD *)sub_37070(a1, v7, v4, v8) + 20;
}

