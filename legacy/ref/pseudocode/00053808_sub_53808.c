/* ea:   0x00053808 */
/* name: sub_53808 */
/* size: 52 */
int __fastcall sub_53808(_DWORD *a1, int a2, int a3)
{
  if ( a2 >= *a1 || a3 >= a1[1] )
    return 0;
  else
    return *(_DWORD *)(*(_DWORD *)(a1[2] + 4 * a2) + 4 * a3);
}

