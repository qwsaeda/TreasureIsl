/* ea:   0x000181C0 */
/* name: sub_181C0 */
/* size: 52 */
int __fastcall sub_181C0(int a1, _DWORD *a2)
{
  int v3; // r3

  v3 = sub_181F4(a1, 1);
  if ( v3 != -16 )
  {
    *(_DWORD *)(v3 + 16) = *a2;
    *(_DWORD *)(v3 + 20) = a2[1];
  }
  return v3;
}

