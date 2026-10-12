/* ea:   0x00066680 */
/* name: sub_66680 */
/* size: 52 */
int __fastcall sub_66680(int a1, _DWORD *a2)
{
  int v3; // r3

  v3 = sub_666B4(a1, 1);
  if ( v3 != -16 )
  {
    *(_DWORD *)(v3 + 16) = *a2;
    *(_DWORD *)(v3 + 20) = a2[1];
  }
  return v3;
}

