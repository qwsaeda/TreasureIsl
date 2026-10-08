/* ea:   0x00065CD8 */
/* name: sub_65CD8 */
/* size: 20 */
void __fastcall sub_65CD8(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

