/* ea:   0x000309EC */
/* name: sub_309EC */
/* size: 68 */
int __fastcall sub_309EC(int a1, _DWORD *a2)
{
  unsigned int v3; // r2

  v3 = *(unsigned __int16 *)(a1 + 12);
  if ( v3 <= 0xA )
  {
    *(_DWORD *)(a1 + 4) = 500;
    *a2 = 500;
    return 500;
  }
  else
  {
    *(_DWORD *)(a1 + 4) = 50 * v3;
    *a2 = 50 * v3;
    return 50 * v3;
  }
}

