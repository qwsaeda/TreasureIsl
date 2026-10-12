/* ea:   0x0005383C */
/* name: sub_5383C */
/* size: 84 */
int __fastcall sub_5383C(_DWORD *a1, int a2)
{
  if ( a2 >= a1[1] * *a1 )
    return 0;
  else
    return *(_DWORD *)(*(_DWORD *)(a1[2] + 4 * (a2 % a1[1])) + 4 * (a2 / a1[1]));
}

