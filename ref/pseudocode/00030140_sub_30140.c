/* ea:   0x00030140 */
/* name: sub_30140 */
/* size: 84 */
int __fastcall sub_30140(int a1, _BYTE *a2)
{
  _BYTE *v4; // r0

  *(_DWORD *)a1 = 0;
  v4 = (_BYTE *)sub_34438();
  *(_DWORD *)(a1 + 4) = 0;
  *(_DWORD *)a1 = v4;
  *(_BYTE *)(a1 + 8) = *a2;
  *v4 = 0;
  *(_DWORD *)(*(_DWORD *)a1 + 4) = 0;
  *(_DWORD *)(*(_DWORD *)a1 + 8) = *(_DWORD *)a1;
  *(_DWORD *)(*(_DWORD *)a1 + 12) = *(_DWORD *)a1;
  return a1;
}

